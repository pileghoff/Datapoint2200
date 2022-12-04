use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

use crate::{
    assembler::assemble,
    clock::Clock,
    cpu::{execute_instruction, fetch_instruction, Cpu},
};

pub struct Datapoint {
    cpu: Cpu,
    clock: Clock,
}

impl Datapoint {
    pub fn new(lines: Vec<&str>, time_scale: f32) -> Datapoint {
        let program = assemble(lines);
        let cpu_clock = channel::<u8>();
        let cpu_intr = channel::<u8>();
        Datapoint {
            cpu: Cpu::new(program, Some(cpu_clock.1), Some(cpu_intr.1)),
            clock: Clock {
                time_scale,
                current_time: 0,
                cpu_clock: cpu_clock.0,
                cpu_intr: cpu_intr.0,
            },
        }
    }

    pub fn run(&mut self) -> usize {
        let mut counter = 0;
        self.clock.run();
        while !self.cpu.halted {
            counter += 1;
            fetch_instruction(&mut self.cpu);
            execute_instruction(&mut self.cpu);
        }

        counter
    }
}
