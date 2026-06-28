#!/usr/bin/env bash
# One-time setup of the ESP32 Rust toolchain for building (and flashing)
# massbee-agent. Installs, idempotently:
#
#   espup     — installs/manages the `esp` Rust toolchain + the Xtensa & RISC-V
#               GCC toolchains (cargo +esp …, and the riscv linker the build
#               needs). Run `espup install` after.
#   ldproxy   — linker shim the *-esp-espidf targets link through (Pitfall 6).
#   espflash  — flash + serial monitor for the built ELF.
#
# After this, build with scripts/esp32-build.sh. See docs/esp32-build.md.
#
# Usage: scripts/esp32-setup.sh
set -euo pipefail

# Install a cargo binary only if it's missing. `cargo install` would no-op on an
# up-to-date crate anyway, but skipping avoids a slow index/compile check.
ensure_cargo_bin() {
  local bin="$1" crate="${2:-$1}"
  if command -v "$bin" >/dev/null 2>&1; then
    echo "== $bin already installed ($("$bin" --version 2>/dev/null | head -1)) =="
  else
    echo "== installing $crate (cargo install) =="
    cargo install "$crate" --locked
  fi
}

# 1. espup — the esp Rust toolchain manager.
ensure_cargo_bin espup espup

# 2. The esp toolchain itself (esp rustc + Xtensa/RISC-V GCC under ~/.espressif).
#    `espup install` is idempotent; re-running just verifies/updates.
if rustup toolchain list 2>/dev/null | grep -q '^esp'; then
  echo "== esp Rust toolchain already present (espup install to update) =="
else
  echo "== espup install (esp toolchain + Xtensa/RISC-V GCC) =="
  espup install
fi
# espup writes an env file you can source for the global esp env; the massbee
# build scripts don't need it (they use a managed ESP-IDF), but flag it.
if [[ -f "$HOME/export-esp.sh" ]]; then
  echo "   (espup env file: ~/export-esp.sh — not required by scripts/esp32-build.sh)"
fi

# 3. ldproxy — required linker shim for the link step.
ensure_cargo_bin ldproxy ldproxy

# 4. espflash — to flash/monitor the board after a build.
ensure_cargo_bin espflash espflash

echo
echo "== setup complete =="
echo "   build:  scripts/esp32-build.sh [--release] [--clean]"
echo "   flash:  scripts/esp32-flash.sh [PORT] [--debug]"
