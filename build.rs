//! Re-export esp-idf-sys's linker arguments to the final binary.
//!
//! esp-idf-sys emits the ESP-IDF static libs, linker scripts, and the
//! `--ldproxy-linker` flag as link args, but they only reach the binary link if
//! the *binary* crate forwards them via `embuild::espidf::sysenv::output()`.
//! Without this, ldproxy fails with `Cannot locate argument '--ldproxy-linker'`.
fn main() {
    embuild::espidf::sysenv::output();

    // Load WIFI_SSID / WIFI_PASS (and any other vars) from a local .env so the
    // `env!(...)` macros in src resolve at compile time without requiring the
    // caller to export them into the shell.
    load_dotenv();
}

/// Read a `.env` file next to Cargo.toml and forward each `KEY=VALUE` entry to
/// rustc as a compile-time env var via `cargo:rustc-env`.
fn load_dotenv() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    println!("cargo:rerun-if-changed={}", path.display());

    let Ok(contents) = std::fs::read_to_string(&path) else {
        return;
    };

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        // Strip optional surrounding quotes from the value.
        let value = value.trim().trim_matches(|c| c == '"' || c == '\'');
        println!("cargo:rustc-env={key}={value}");
    }
}
