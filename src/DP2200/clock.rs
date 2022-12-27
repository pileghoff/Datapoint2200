use crate::time::Instant;
use std::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct Clock {
    pub time_scale: f32,
    pub emulated_time_ns: u128,
    pub cpu_intr: Sender<u8>,
    pub databus_clock: Sender<u8>,
    pub last_time: Instant,
}

const CYCLE_TIME_NS: u128 = 1_600;
const INTR_TIME_NS: u128 = 1_000_000;
// 153.6kHz clock @ 1600 ns
const DATABUS_CLOCK_NS: u128 = 156_300 / CYCLE_TIME_NS;

impl Clock {
    fn check_trigger(&self, trigger_time: u128, cycles: u128) -> bool {
        if cycles * CYCLE_TIME_NS > trigger_time {
            return true;
        }

        (self.emulated_time_ns % trigger_time)
            > ((self.emulated_time_ns + CYCLE_TIME_NS * cycles) % trigger_time)
    }

    pub fn single_clock(&mut self) {
        self.ticks(1);
    }

    pub fn ticks(&mut self, num_clocks: u128) {
        if self.check_trigger(INTR_TIME_NS, num_clocks) {
            self.cpu_intr.send(0).unwrap();
        }

        if self.check_trigger(DATABUS_CLOCK_NS, num_clocks) {
            self.databus_clock.send(0).unwrap();
        }

        self.emulated_time_ns += CYCLE_TIME_NS * num_clocks;
    }
}
