"""Adds the START A BIT board and its page messages to an esp32sim checkout: python3 patch/apply.py <esp32sim folder>
  - esp32/src/board/startabit.rs: the board (patch/startabit.rs), named "startabit" in esp32/src/board/mod.rs
  - esp-soc/src/board.rs: BoardModel::page_message (a no-op unless a board takes it), appended at the end of the trait
  - esp-soc/src/machine/web.rs: {"t":"gpio","pin":N,"level":0|1} sets an input pin, {"t":"adc","pin":N,"raw":0-4095} an ADC
    count, {"t":"sonar","trig":N,"echo":N} and {"t":"distance","cm":X} the distance sensor (v2); "key" with src "uart2"
    types on UART2 (v2: an HM-10 on UART2)
"""
import os
import shutil
import sys

ROOT = sys.argv[1]
HERE = os.path.dirname(os.path.abspath(__file__))


def swap(path, old, new):
    path = os.path.join(ROOT, path)
    text = open(path, encoding='utf-8').read()
    if text.count(old) != 1:
        sys.exit('patch: %r is not once in %s' % (old[:60], path))
    open(path, 'w', encoding='utf-8', newline='\n').write(text.replace(old, new))


shutil.copy(os.path.join(HERE, 'startabit.rs'), os.path.join(ROOT, 'esp32', 'src', 'board', 'startabit.rs'))
swap('esp32/src/board/mod.rs', 'pub mod ulanzi;\n', 'pub mod ulanzi;\npub mod startabit;\n')
swap('esp32/src/board/mod.rs', '        "ulanzi-tc001" | "tc001" => Some(Box::new(ulanzi::UlanziTc001::new())),\n',
     '        "ulanzi-tc001" | "tc001" => Some(Box::new(ulanzi::UlanziTc001::new())),\n'
     '        "startabit" => Some(Box::new(startabit::StartABit::default())),\n')
swap('esp-soc/src/board.rs', '    fn camera_frame_is_jpeg(&self) -> bool { false }\n}\n',
     '    fn camera_frame_is_jpeg(&self) -> bool { false }\n'
     '    /// A message from the page for the board (the START A BIT Code lab: its distance sensor). Appended last.\n'
     '    fn page_message(&mut self, _kind: &str, _a: f64, _b: f64) {}\n}\n')
swap('esp-soc/src/machine/web.rs', '                        Some("uart1") => self.bus.uart_input(1, data.as_bytes()),\n',
     '                        Some("uart1") => self.bus.uart_input(1, data.as_bytes()),\n'
     '                        Some("uart2") => self.bus.uart_input(2, data.as_bytes()),\n')
swap('esp-soc/src/machine/web.rs', '                _ => {}\n            }\n        }\n    }\n',
     '                // the START A BIT Code lab (startabit/esp32sim-wasm): an input pin\'s level, an ADC pin\'s raw count,\n'
     '                // the distance sensor\'s pins and how far away the thing is\n'
     '                "gpio" => { let pin: u8 = field("pin").and_then(|x| x.parse().ok()).unwrap_or(0); let level = field("level").unwrap_or_default() == "1";\n'
     '                            self.bus.gpio_set_input(pin, level); *self.bus.irq_dirty() = true; }\n'
     '                "adc" => { let pin: u8 = field("pin").and_then(|x| x.parse().ok()).unwrap_or(0); let raw: u16 = field("raw").and_then(|x| x.parse().ok()).unwrap_or(0);\n'
     '                           self.bus.adc_set_raw(pin, raw); }\n'
     '                "sonar" => { let trig: f64 = field("trig").and_then(|x| x.parse().ok()).unwrap_or(0.0); let echo: f64 = field("echo").and_then(|x| x.parse().ok()).unwrap_or(0.0);\n'
     '                             self.bus.board().page_message("sonar", trig, echo); }\n'
     '                "distance" => { let cm: f64 = field("cm").and_then(|x| x.parse().ok()).unwrap_or(100.0); self.bus.board().page_message("distance", cm, 0.0); }\n'
     '                _ => {}\n            }\n        }\n    }\n')
print('patched', ROOT)
