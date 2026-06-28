#!/usr/bin/env bash
# Flash the built massbee-agent ELF to a connected ESP32-C3 and open the serial
# monitor. Thin wrapper around `espflash flash` (espflash 4.x).
#
# Gotcha this script removes: in espflash 4.x the ELF is the *only* positional
# argument; the port is `-p/--port`. Passing the port positionally makes
# espflash treat it as the image and reject the real ELF as an unexpected arg.
#
# Usage:
#   scripts/esp32-flash.sh [PORT] [--debug] [extra espflash args…]
#
#   PORT      serial device (default: auto-detect by espflash). e.g.
#             /dev/cu.usbmodem101 on macOS, /dev/ttyUSB0 / /dev/ttyACM0 on Linux.
#   --debug   flash the debug ELF instead of release (default: release).
#   Anything else is passed straight through to `espflash flash`.
#
# Examples:
#   scripts/esp32-flash.sh                              # release, auto-port
#   scripts/esp32-flash.sh /dev/cu.usbmodem101          # release, explicit port
#   scripts/esp32-flash.sh /dev/cu.usbmodem101 --debug  # debug build
set -euo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"

TARGET="riscv32imc-esp-espidf"
PROFILE="release"
PORT=""
EXTRA=()

for arg in "$@"; do
  case "$arg" in
    --debug)   PROFILE="debug" ;;
    --release) PROFILE="release" ;;
    /dev/*)    PORT="$arg" ;;            # serial device path
    *)         EXTRA+=("$arg") ;;        # pass-through to espflash
  esac
done

ELF="target/$TARGET/$PROFILE/esp32c3-hub"
if [[ ! -f "$ELF" ]]; then
  echo "error: $ELF not found — build it first:" >&2
  echo "  scripts/esp32-build.sh$([[ $PROFILE == release ]] && echo ' --release')" >&2
  exit 1
fi

# Port is a *flag* (-p); the ELF is the sole positional and must come last.
PORT_ARGS=()
[[ -n "$PORT" ]] && PORT_ARGS=(--port "$PORT")

echo "== flashing $ELF (chip esp32c3${PORT:+, port $PORT}) =="
espflash flash --monitor --chip esp32c3 "${PORT_ARGS[@]}" "${EXTRA[@]}" "$ELF"
