"""Adds the START A BIT board and two page messages to an esp32sim checkout: python3 patch/apply.py <esp32sim folder>
  - esp32/src/board/startabit.rs: the board (patch/startabit.rs), named "startabit" in esp32/src/board/mod.rs
  - esp-soc/src/machine/web.rs: {"t":"gpio","pin":N,"level":0|1} sets an input pin, {"t":"adc","pin":N,"raw":0-4095} an ADC count
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
swap('esp-soc/src/machine/web.rs', '                _ => {}\n            }\n        }\n    }\n',
     '                // the START A BIT Code lab (startabit/esp32sim-wasm): an input pin\'s level, an ADC pin\'s raw count\n'
     '                "gpio" => { let pin: u8 = field("pin").and_then(|x| x.parse().ok()).unwrap_or(0); let level = field("level").unwrap_or_default() == "1";\n'
     '                            self.bus.gpio_set_input(pin, level); *self.bus.irq_dirty() = true; }\n'
     '                "adc" => { let pin: u8 = field("pin").and_then(|x| x.parse().ok()).unwrap_or(0); let raw: u16 = field("raw").and_then(|x| x.parse().ok()).unwrap_or(0);\n'
     '                           self.bus.adc_set_raw(pin, raw); }\n'
     '                _ => {}\n            }\n        }\n    }\n')
print('patched', ROOT)
