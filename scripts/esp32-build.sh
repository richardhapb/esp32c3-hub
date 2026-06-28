#!/usr/bin/env bash
# Clean cross-compile of massbee-agent for ESP32-C3 (RISC-V / ESP-IDF).
# Usage:
#   scripts/esp32-build.sh                 # debug build
#   scripts/esp32-build.sh --release       # release build
#   scripts/esp32-build.sh --clean         # wipe esp-idf-sys CMake cache first
#   (extra args are passed through to `cargo build`)
set -euo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"

TARGET="riscv32imc-esp-espidf"

# --clean: drop the per-target esp-idf-sys build dir so CMake reconfigures from
# scratch (needed if a previous build used a different IDF version/source).
CARGO_ARGS=()
for arg in "$@"; do
  if [[ "$arg" == "--clean" ]]; then
    echo "== wiping esp-idf-sys build cache =="
    rm -rf "target/$TARGET"/*/build/esp-idf-sys-* 2>/dev/null || true
  else
    CARGO_ARGS+=("$arg")
  fi
done

echo "== building massbee-agent for $TARGET (managed ESP-IDF v5.4) =="
env -u IDF_PATH -u VIRTUAL_ENV -u IDF_PYTHON_ENV_PATH \
    -u ESP_IDF_TOOLS_INSTALL_DIR -u PYTHONHOME -u PYTHONPATH \
  cargo +esp build -Z build-std=std,panic_abort \
    --target "$TARGET" "${CARGO_ARGS[@]}"

echo "== done — artifact: target/$TARGET/{debug,release}/massbee-agent =="
