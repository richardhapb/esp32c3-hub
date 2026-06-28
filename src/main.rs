use esp_idf_svc::log::init_from_env;
use log::info;

fn main() {
    init_from_env();
    info!("DONE!!!!");
}
