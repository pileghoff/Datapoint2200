use core::time;
use std::sync::mpsc::Sender;

use std::thread;
use std::thread::JoinHandle;

#[derive(Debug, Clone)]
pub struct Clock {
    pub time_scale: f32,
    pub current_time: usize,
    pub cpu_clock: Sender<u8>,
    pub cpu_intr: Sender<u8>,
    pub databus_clock: Sender<u8>,
}

const CYCLE_TIME_NS: usize = 1_600;
const INTR_TIME_NS: usize = 1_000_000;
// 153.6kHz clock @ 1600 ns
const DATABUS_CLOCK_NS: usize = 156_300 / CYCLE_TIME_NS;

impl Clock {
    fn check_trigger(&self, trigger_time: usize) -> bool {
        (self.current_time % trigger_time) > ((self.current_time + CYCLE_TIME_NS) % trigger_time)
    }

    pub fn run(mut self) -> JoinHandle<Clock> {
        thread::spawn(move || {
            let to_sleep = time::Duration::new(0, (CYCLE_TIME_NS as f32 * self.time_scale) as u32);

            loop {
                thread::sleep(to_sleep);
                let running = self.cpu_clock.send(0);
                if running.is_err() {
                    break;
                }

                if self.check_trigger(INTR_TIME_NS) {
                    self.cpu_intr.send(0).unwrap();
                }

                if self.check_trigger(DATABUS_CLOCK_NS) {
                    self.databus_clock.send(0).unwrap();
                }

                self.current_time += CYCLE_TIME_NS;
            }

            self
        })
    }
}
