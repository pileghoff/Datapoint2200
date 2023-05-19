use super::{cpu::Cpu, databus::Databus};

#[derive(Debug, Clone)]
pub struct Clock {
    pub time_scale: f32,
    pub emulated_time_ns: u128,
}

const CYCLE_TIME_NS: u128 = 1_600;
const INTR_TIME_NS: u128 = 1_000_000;
// 153.6kHz clock @ 1600 ns
const DATABUS_CLOCK_NS: u128 = 156_300 / CYCLE_TIME_NS;

impl Clock {
    pub fn build(time_scale: f32) -> Clock {
        Clock {
            time_scale,
            emulated_time_ns: 0,
        }
    }

    fn check_trigger(&self, trigger_time: u128, cycles: u128) -> bool {
        if cycles * CYCLE_TIME_NS > trigger_time {
            return true;
        }

        (self.emulated_time_ns % trigger_time)
            > ((self.emulated_time_ns + CYCLE_TIME_NS * cycles) % trigger_time)
    }

    pub fn single_clock(&mut self, cpu: &mut Cpu, databus: &mut Databus) {
        self.ticks(1, cpu, databus);
    }

    pub fn ticks(&mut self, num_clocks: u128, cpu: &mut Cpu, databus: &mut Databus) {
        if self.check_trigger(INTR_TIME_NS, num_clocks) {
            cpu.interrupt();
        }

        if self.check_trigger(DATABUS_CLOCK_NS, num_clocks) {
            databus.clock();
        }

        self.emulated_time_ns += CYCLE_TIME_NS * num_clocks;
    }
}
