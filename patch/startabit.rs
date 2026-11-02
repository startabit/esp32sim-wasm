//! The START A BIT Code lab's board (startabit/esp32sim-wasm): a classic ESP32 whose pins the page draws with the
//! parts wired to them. Every 20 ms of emulated time it reports each GPIO as one entry of the LED grid `pins`:
//! [255 when HIGH else 0, the share of the time HIGH (0-255), the last HIGH pulse in 10 µs steps (0-255)], so the
//! page shows a level, a PWM brightness and a servo's pulse. A pin the LEDC drives (PWM) is reported from its routed
//! frequency and duty, and the grid `pwm` gives [frequency low byte, high byte, duty 0-255] for each pin (v2). WS2812
//! frames (RMT) become the grid `strip`. The page sets input pins and ADC counts with the `gpio` and `adc` messages;
//! `sonar` {trig, echo} and `distance` {cm} make an HC-SR04 answer a Trig pulse on its Echo pin (v2).
use esp_soc::BoardModel;
use esp_soc::board::BoardEdge;

const PINS: usize = 40;
const HZ: u64 = 240_000_000;              // the bus reference clock of the classic ESP32 model
const WINDOW: u64 = HZ / 50;              // 20 ms
const US: u64 = HZ / 1_000_000;           // cycles in a microsecond

pub struct StartABit {
    level: [bool; PINS],
    since: [u64; PINS],
    high: [u64; PINS],
    rise: [u64; PINS],
    pulse: [u64; PINS],
    pwm: [Option<(f64, u32)>; PINS],
    window_start: u64,
    grid: Vec<[u8; 3]>,
    pwm_grid: Vec<[u8; 3]>,
    updates: u64,
    strip: Vec<[u8; 3]>,
    strip_updates: u64,
    events: u64,
    // the distance sensor: its pins, how far the thing in front of it is, and the Echo edges still to come
    sonar: Option<(u8, u8)>,
    cm: f64,
    coming: Vec<(u64, u8, bool)>,
    edges: Vec<BoardEdge>,
}

impl Default for StartABit {
    fn default() -> Self {
        StartABit { level: [false; PINS], since: [0; PINS], high: [0; PINS], rise: [0; PINS], pulse: [0; PINS], pwm: [None; PINS],
                    window_start: 0, grid: vec![[0, 0, 0]; PINS], pwm_grid: vec![[0, 0, 0]; PINS], updates: 0, strip: Vec::new(),
                    strip_updates: 0, events: 0, sonar: None, cm: 100.0, coming: Vec::new(), edges: Vec::new() }
    }
}

impl StartABit {
    fn close_window(&mut self, cycle: u64) {
        let span = cycle.saturating_sub(self.window_start).max(1);
        for p in 0..PINS {
            let mut high = self.high[p];
            if self.level[p] { high += cycle.saturating_sub(self.since[p]); self.since[p] = cycle; }
            self.high[p] = 0;
            if let Some((hz, duty)) = self.pwm[p] {          // the LEDC drives it: its own frequency and duty
                let share = (duty as u64 * 255 / 65535) as u8;
                let pulse = ((duty as f64 / 65535.0) / hz * 100_000.0).round().min(255.0) as u8;
                self.grid[p] = [if duty > 0 { 255 } else { 0 }, share, pulse];
                let f = hz.round().min(65535.0) as u32;
                self.pwm_grid[p] = [(f & 255) as u8, (f >> 8) as u8, share];
                continue;
            }
            self.pwm_grid[p] = [0, 0, 0];
            let share = ((high as u128 * 255) / span as u128).min(255) as u8;
            let pulse = (self.pulse[p] / (HZ / 100_000)).min(255) as u8;
            self.grid[p] = [if self.level[p] { 255 } else { 0 }, share, pulse];
        }
        self.window_start = cycle;
        self.updates += 1;
    }
    // the Echo edges that are due by now
    fn edges_to(&mut self, cycle: u64) {
        self.coming.sort_by_key(|e| e.0);
        while let Some(&(at, pin, level)) = self.coming.first() {
            if at > cycle { break; }
            self.coming.remove(0);
            self.edges.push(BoardEdge { cycle: at, pin, level });
        }
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
            // Trig HIGH for at least 8 µs, then LOW: 450 µs later Echo is HIGH for as long as the sound takes there and back
            if let Some((trig, echo)) = self.sonar {
                if pin == trig && !level && self.pulse[p] >= 8 * US {
                    let echo_us = if self.cm <= 0.0 || self.cm > 400.0 { 38_000.0 } else { self.cm * 2.0 / 0.0343 };
                    let start = cycle + 450 * US;
                    self.coming.push((start, echo, true));
                    self.coming.push((start + (echo_us * US as f64) as u64, echo, false));
                }
            }
        }
    }
    fn gpio_events(&self) -> u64 { self.events }
    fn next_deadline(&self) -> Option<u64> {
        let window = self.window_start + WINDOW;
        Some(self.coming.iter().map(|e| e.0).min().map_or(window, |at| at.min(window)))
    }
    fn advance_to(&mut self, cycle: u64) {
        self.edges_to(cycle);
        if cycle >= self.window_start + WINDOW { self.close_window(cycle); }
    }
    // every step: the Echo edges; every 20 ms the pins, with the PWM the GPIO matrix routes to each one
    fn advance_inputs_with_clock(&mut self, cycle: u64, clock: &dyn Fn(u8) -> Option<(f64, u32)>) {
        self.edges_to(cycle);
        if cycle >= self.window_start + WINDOW {
            for p in 0..PINS { self.pwm[p] = clock(p as u8).filter(|&(hz, _)| hz.is_finite() && hz > 0.0); }
            self.close_window(cycle);
        }
    }
    fn take_edges(&mut self) -> Vec<BoardEdge> { std::mem::take(&mut self.edges) }
    fn rmt_frame(&mut self, _pin: u8, bits: &[bool]) {
        let leds: Vec<[u8; 3]> = bits.chunks_exact(24).map(|led| {
            let byte = |k: usize| led[k * 8..k * 8 + 8].iter().fold(0u8, |v, &b| (v << 1) | b as u8);
            [byte(1), byte(0), byte(2)]                                   // sent green, red, blue
        }).collect();
        if !leds.is_empty() { self.strip = leds; self.strip_updates += 1; }
    }
    fn led_grids(&self) -> Vec<(&'static str, &[[u8; 3]], u64)> {
        let mut grids = vec![("pins", &self.grid[..], self.updates), ("pwm", &self.pwm_grid[..], self.updates)];
        if !self.strip.is_empty() { grids.push(("strip", &self.strip[..], self.strip_updates)); }
        grids
    }
    fn page_message(&mut self, kind: &str, a: f64, b: f64) {
        match kind {
            "sonar" => self.sonar = Some((a as u8, b as u8)),
            "distance" => self.cm = a,
            _ => {}
        }
    }
}
