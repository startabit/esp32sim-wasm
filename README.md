# esp32sim for the browser

A simulated ESP32 for the Code lab of [startabit.com](https://www.startabit.com/code-lab/): students' MicroPython runs
on the official MicroPython firmware, on an emulated classic ESP32, in the browser.

The emulator is [esp32sim](https://github.com/mikroverk/esp32sim) (MIT), built here as WebAssembly at a pinned
commit with one addition ([patch/](patch/)): a board named `startabit` that reports the pins and LED strips to the
page as LED grids, and the page messages `gpio` and `adc` for input pins and analog values.

## A release

| File | What |
|---|---|
| `esp32sim.wasm.gz` | the emulator, `tools/wasm-build.sh` of esp32sim with the patch |
| `esp32_rev300_rom.elf.gz` | the classic ESP32 mask ROM, from [espressif/esp-rom-elfs](https://github.com/espressif/esp-rom-elfs) (Apache-2.0) |
| `micropython-esp32-v1.29.0.bin.gz` | MicroPython 1.29.0 for the ESP32 (ESP32_GENERIC, [micropython.org](https://micropython.org/download/ESP32_GENERIC/), MIT), written at 0x1000 |
| `esp32sim-source.tar.gz`, `startabit.patch` | the source it was built from, and our change |

Built by [.github/workflows/build.yml](.github/workflows/build.yml); downloads are checked against SHA-256 sums.
Our patch is under the MIT licence, like esp32sim (its licence is `LICENSE-esp32sim` in each release).
