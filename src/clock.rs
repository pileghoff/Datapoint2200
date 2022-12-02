use std::{
    thread,
    time::{self, Instant},
};

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    pub time_scale: f32,
    pub current_time: usize,
}

const INTR_TIME: usize = 1_000_000;

impl Clock {
    pub fn new(time_scale: f32) -> Clock {
        Clock {
            time_scale,
            current_time: 0,
        }
    }

    pub fn clock(&mut self, cycles: usize) -> bool {
        let elapsed_time_nano = cycles * 1600;
        let elapsed_time_scaled = (((cycles * 1600) as f32) * self.time_scale) as u128;
        let duration = time::Duration::new(0, elapsed_time_scaled as u32);
        thread::sleep(duration);
        let intr_trigger =
            (self.current_time % INTR_TIME) > (self.current_time + elapsed_time_nano) % INTR_TIME;

        self.current_time += elapsed_time_nano;

        intr_trigger
    }
}
