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
    pub fn run(&mut self) -> JoinHandle<()> {
        let mut clock = self.clone();
        thread::spawn(move || {
            let to_sleep = time::Duration::new(0, (CYCLE_TIME_NS as f32 * clock.time_scale) as u32);

            loop {
                thread::sleep(to_sleep);
                let running = clock.cpu_clock.send(0);
                if running.is_err() {
                    break;
                }

                let intr_trigger = (clock.current_time % INTR_TIME_NS)
                    > (clock.current_time + CYCLE_TIME_NS) % INTR_TIME_NS;
                if intr_trigger {
                    clock.cpu_intr.send(0);
                }

                let databus_trigger = (clock.current_time % DATABUS_CLOCK_NS)
                    > (clock.current_time + CYCLE_TIME_NS) % DATABUS_CLOCK_NS;
                if databus_trigger {
                    clock.databus_clock.send(0);
                }

                clock.current_time += CYCLE_TIME_NS;
            }
        })
    }
}
