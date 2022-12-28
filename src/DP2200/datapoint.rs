use log::{error, info, trace, warn};

use crate::time::Instant;
use std::sync::mpsc::channel;

use crate::DP2200::{
    assembler::assemble,
    clock::Clock,
    cpu::Cpu,
    databus::{Databus, DatabusMode, Dataline},
    screen::Screen,
};

use super::{instruction::Instruction, keyboard::Keyboard};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
pub enum DataPointRunStatus {
    Ok,
    BreakpointHit,
    Halted,
}

#[derive(Debug)]
pub struct Datapoint {
    pub cpu: Cpu,
    pub clock: Clock,
    pub databus: Databus,
    pub breakpoints: Vec<u16>,
}

impl Datapoint {
    pub fn build(program: &[u8], time_scale: f32) -> Datapoint {
        let cpu_intr = channel::<u8>();
        let databus_clock = channel::<u8>();
        let dataline = Dataline::generate_pair();
        let mut res = Datapoint {
            breakpoints: Vec::new(),
            cpu: Cpu {
                halted: false,
                intr_enabled: false,
                intr_saved: false,
                memory: [0; 4096],
                alpha_mode: true,
                alpha_registers: [0, 0, 0, 0, 0, 0, 0],
                alpha_flipflops: [false, false, false, false],
                beta_registers: [0, 0, 0, 0, 0, 0, 0],
                beta_flipflops: [false, false, false, false],
                program_counter: 0,
                stack: Vec::new(),
                intr: cpu_intr.1,
                dataline: dataline.0,
                instruction_register: Instruction::unknown(),
            },
            clock: Clock {
                time_scale,
                emulated_time_ns: 0,
                cpu_intr: cpu_intr.0,
                databus_clock: databus_clock.0,
                last_time: Instant::now(),
            },
            databus: Databus {
                selected_addr: 0,
                selected_mode: DatabusMode::Status,
                clock: databus_clock.1,
                dataline: dataline.1,
                screen: Screen::new(),
                keyboard: Keyboard::new(),
            },
        };

        for (i, b) in program.iter().enumerate() {
            res.cpu.memory[i] = *b;
        }

        res
    }

    pub fn new(lines: Vec<&str>, time_scale: f32) -> Datapoint {
        let program = assemble(lines).unwrap();
        Datapoint::build(&program, time_scale)
    }

    pub fn load_program(&mut self, program: &[u8]) {
        for i in 0..self.cpu.memory.len() {
            if let Some(byte) = program.get(i) {
                self.cpu.memory[i] = *byte;
            } else {
                self.cpu.memory[i] = 0;
            }
        }

        self.breakpoints = Vec::new();
    }

    pub fn update(&mut self, delta_time_ms: f64) -> DataPointRunStatus {
        if self.cpu.halted {
            trace!("Total execution time: {}", self.clock.emulated_time_ns);
            return DataPointRunStatus::Halted;
        }

        let goal_time = self.clock.emulated_time_ns + (delta_time_ms * 1_000_000.0) as u128;

        loop {
            let inst = self.cpu.fetch_instruction();
            if inst.is_none() {
                error!(
                    "Could not fetch instruction. Cpu program counter: {}",
                    self.cpu.program_counter
                );
                return DataPointRunStatus::Halted;
            }
            self.cpu.instruction_register = inst.unwrap();

            if self.breakpoints.contains(&self.cpu.program_counter) {
                return DataPointRunStatus::BreakpointHit;
            }

            self.clock
                .ticks(self.cpu.instruction_register.get_clock_cycles() as u128);

            if !self.cpu.execute_instruction() {
                break;
            }

            self.databus.run();

            if self.clock.emulated_time_ns >= goal_time {
                break;
            }
        }

        DataPointRunStatus::Ok
    }

    pub fn single_step(&mut self) -> DataPointRunStatus {
        loop {
            self.clock.single_clock();
            if self.cpu.execute_instruction() {
                break;
            }
        }

        self.databus.run();

        if self.cpu.halted {
            return DataPointRunStatus::Halted;
        }

        DataPointRunStatus::Ok
    }

    pub fn run(&mut self) -> u128 {
        while !self.cpu.halted {
            self.update(10.0);
        }

        return self.clock.emulated_time_ns;
    }

    pub fn toggle_breakpoint(&mut self, addr: u16) {
        if let Some(index) = self.breakpoints.iter().position(|&x| addr == x) {
            self.breakpoints.remove(index);
        } else {
            self.breakpoints.push(addr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_addr() {
        let program = vec!["LoadImm A, 0x69", "Adr", "Halt"];

        let mut machine = Datapoint::new(program, 1.0);
        machine.run();
        let db = machine.databus;
        assert_eq!(db.selected_addr, 0x69);
        assert_eq!(db.selected_mode, DatabusMode::Status);
    }

    #[test]
    fn test_write_to_screen() {
        let program = vec!["LoadImm A, 0xe1", "Adr", "LoadImm A, 0x5a", "Write", "Halt"];

        let mut machine = Datapoint::new(program, 1.0);
        machine.run();
        let db = machine.databus;
        assert_eq!(db.selected_addr, 0xe1);
        assert_eq!(db.screen.buffer[0][0], 'Z');
    }
}
