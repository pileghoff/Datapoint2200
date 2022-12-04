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
}

const INTR_TIME: usize = 1_000_000;

impl Clock {
    pub fn run(&mut self) -> JoinHandle<()> {
        let mut clock = self.clone();
        thread::spawn(move || {
            let to_sleep = time::Duration::new(0, (1600.0 * clock.time_scale) as u32);

            loop {
                thread::sleep(to_sleep);
                let running = clock.cpu_clock.send(0);
                if running.is_err() {
                    break;
                }

                let intr_trigger =
                    (clock.current_time % INTR_TIME) > (clock.current_time + 1600) % INTR_TIME;
                if intr_trigger {
                    clock.cpu_intr.send(0);
                }

                clock.current_time += 1600;
            }
        })
    }
}
