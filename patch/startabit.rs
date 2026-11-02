//! The START A BIT Code lab's board (startabit/esp32sim-wasm): a classic ESP32 whose pins the page draws with the
//! parts wired to them. Every 20 ms of emulated time it reports each GPIO as one entry of the LED grid `pins`:
//! [255 when HIGH else 0, the share of the time HIGH (0-255), the last HIGH pulse in 10 µs steps (0-255)], so the
//! page shows a level, a PWM brightness and a servo's pulse. WS2812 frames (RMT) become the grid `strip`. The page
//! sets input pins and ADC counts with the `gpio` and `adc` messages.
use esp_soc::BoardModel;

const PINS: usize = 40;
const HZ: u64 = 240_000_000;              // the bus reference clock of the classic ESP32 model
const WINDOW: u64 = HZ / 50;              // 20 ms

pub struct StartABit {
    level: [bool; PINS],
    since: [u64; PINS],
    high: [u64; PINS],
    rise: [u64; PINS],
    pulse: [u64; PINS],
    window_start: u64,
    grid: Vec<[u8; 3]>,
    updates: u64,
    strip: Vec<[u8; 3]>,
    strip_updates: u64,
    events: u64,
}

impl Default for StartABit {
    fn default() -> Self {
        StartABit { level: [false; PINS], since: [0; PINS], high: [0; PINS], rise: [0; PINS], pulse: [0; PINS], window_start: 0,
                    grid: vec![[0, 0, 0]; PINS], updates: 0, strip: Vec::new(), strip_updates: 0, events: 0 }
    }
}

impl StartABit {
    fn close_window(&mut self, cycle: u64) {
        let span = cycle.saturating_sub(self.window_start).max(1);
        for p in 0..PINS {
            let mut high = self.high[p];
            if self.level[p] { high += cycle.saturating_sub(self.since[p]); self.since[p] = cycle; }
            self.high[p] = 0;
            let share = ((high as u128 * 255) / span as u128).min(255) as u8;
            let pulse = (self.pulse[p] / (HZ / 100_000)).min(255) as u8;
            self.grid[p] = [if self.level[p] { 255 } else { 0 }, share, pulse];
        }
        self.window_start = cycle;
        self.updates += 1;
    }
}

impl BoardModel for StartABit {
    fn name(&self) -> &'static str { "startabit" }
    fn gpio_output_at(&mut self, cycle: u64, changes: &[(u8, bool)], _enabled: u64, _output: u64) {
        for &(pin, level) in changes {
            let p = pin as usize;
            if p >= PINS || self.level[p] == level { continue; }
            self.events += 1;
            if self.level[p] { self.high[p] += cycle.saturating_sub(self.since[p]); }
            if level { self.rise[p] = cycle; } else { self.pulse[p] = cycle.saturating_sub(self.rise[p]); }
            self.level[p] = level;
            self.since[p] = cycle;
        }
        if cycle >= self.window_start + WINDOW { self.close_window(cycle); }
    }
    fn gpio_events(&self) -> u64 { self.events }
    fn next_deadline(&self) -> Option<u64> { Some(self.window_start + WINDOW) }
    fn advance_to(&mut self, cycle: u64) { if cycle >= self.window_start + WINDOW { self.close_window(cycle); } }
    fn rmt_frame(&mut self, _pin: u8, bits: &[bool]) {
        let leds: Vec<[u8; 3]> = bits.chunks_exact(24).map(|led| {
            let byte = |k: usize| led[k * 8..k * 8 + 8].iter().fold(0u8, |v, &b| (v << 1) | b as u8);
            [byte(1), byte(0), byte(2)]                                   // sent green, red, blue
        }).collect();
        if !leds.is_empty() { self.strip = leds; self.strip_updates += 1; }
    }
    fn led_grids(&self) -> Vec<(&'static str, &[[u8; 3]], u64)> {
        let mut grids = vec![("pins", &self.grid[..], self.updates)];
        if !self.strip.is_empty() { grids.push(("strip", &self.strip[..], self.strip_updates)); }
        grids
    }
}
