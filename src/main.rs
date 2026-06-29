use core::convert::TryInto;
use esp_idf_svc::sys::EspError;
use esp_idf_svc::{hal::gpio::PinDriver, http::Method, io::Write};
use std::sync::{Arc, Mutex};

use esp_idf_svc::hal::gpio::Gpio3;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::log::init_from_env;
use esp_idf_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop, http::server::EspHttpServer, nvs::EspDefaultNvsPartition,
};

use log::{error, info, warn};

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");
const STACK_SIZE: usize = 10240;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    init_from_env();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    connect_wifi(&mut wifi)?;
    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("Wifi DHCP info: {ip_info:?}");

    let _server = run_server(peripherals.pins.gpio3)?;

    loop {
        std::thread::sleep(core::time::Duration::from_secs(3600));
    }
}

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        auth_method: AuthMethod::WPA3Personal,
        password: PASSWORD.try_into().unwrap(),
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;
    wifi.start()?;
    info!("Wifi started");

    with_retry(|| wifi.connect(), 100)?;
    info!("Wifi connected");

    with_retry(|| wifi.wait_netif_up(), 100)?;
    info!("Wifi netif up");

    Ok(())
}

fn with_retry<F>(mut func: F, max_attempts: usize) -> Result<(), EspError>
where
    F: FnMut() -> Result<(), EspError>,
{
    const RETRY_INTERVAL_SEC: u64 = 5;
    let mut attempt = 1;

    while let Err(e) = func() {
        if attempt > max_attempts {
            error!("max attempts reached, dropping connection");
            error!("{e}");
            return Err(e);
        }
        warn!(
            "error connecting to wifi: {e}, attempt {attempt}/{max_attempts}, retrying in {RETRY_INTERVAL_SEC}s"
        );
        attempt += 1;
        std::thread::sleep(std::time::Duration::from_secs(RETRY_INTERVAL_SEC));
        info!("Retrying...");
    }

    Ok(())
}

fn run_server(gp: Gpio3<'static>) -> anyhow::Result<EspHttpServer<'static>> {
    let mut server = create_server()?;

    // Drive the pin as a plain output and track the logical state ourselves.
    // Reading the pad back (is_high) isn't reliable, so we keep `on` as the
    // source of truth instead of inferring the state from the pin.
    let mut driver = PinDriver::output(gp).expect("get pin output");

    driver.set_low()?;
    let state = Arc::new(Mutex::new((driver, false)));

    server.fn_handler("/toggle-lamp", Method::Get, move |req| {
        let mut state = state.lock().unwrap();
        let (driver, on) = &mut *state;
        *on = !*on;
        if *on {
            driver.set_high().ok();
        } else {
            driver.set_low().ok();
        }
        let msg: &[u8] = if *on { b"Lamp on" } else { b"Lamp off" };
        req.into_ok_response()
            .unwrap()
            .write_all(msg)
            .map_err(|_| ())
    })?;

    Ok(server)
}

fn create_server() -> anyhow::Result<EspHttpServer<'static>> {
    let server_configuration = esp_idf_svc::http::server::Configuration {
        stack_size: STACK_SIZE,
        ..Default::default()
    };

    Ok(EspHttpServer::new(&server_configuration)?)
}
