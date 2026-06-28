//! Re-export esp-idf-sys's linker arguments to the final binary.
//!
//! esp-idf-sys emits the ESP-IDF static libs, linker scripts, and the
//! `--ldproxy-linker` flag as link args, but they only reach the binary link if
//! the *binary* crate forwards them via `embuild::espidf::sysenv::output()`.
//! Without this, ldproxy fails with `Cannot locate argument '--ldproxy-linker'`.
fn main() {
    embuild::espidf::sysenv::output();
}
