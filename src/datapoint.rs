use std::sync::mpsc::channel;

use crate::{
    assembler::assemble,
    clock::Clock,
    cpu::Cpu,
    databus::{Databus, DatabusMode, Dataline},
    screen::Screen,
};

pub struct Datapoint {
    pub cpu: Cpu,
    pub clock: Option<Clock>,
    pub databus: Option<Databus>,
}

impl Datapoint {
    pub fn new(lines: Vec<&str>, time_scale: f32) -> Datapoint {
        let program = assemble(lines);
        let cpu_clock = channel::<u8>();
        let cpu_intr = channel::<u8>();
        let databus_clock = channel::<u8>();
        let dataline = Dataline::generate_pair();
        let mut res = Datapoint {
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
                clock: cpu_clock.1,
                intr: cpu_intr.1,
                dataline: dataline.0,
            },
            clock: Some(Clock {
                time_scale,
                current_time: 0,
                cpu_clock: cpu_clock.0,
                cpu_intr: cpu_intr.0,
                databus_clock: databus_clock.0,
            }),
            databus: Some(Databus {
                selected_addr: 0,
                selected_mode: DatabusMode::Status,
                clock: databus_clock.1,
                dataline: dataline.1,
                screen: Screen::new(),
            }),
        };

        for (i, b) in program.iter().enumerate() {
            res.cpu.memory[i] = *b;
        }

        res
    }

    pub fn run(&mut self) -> usize {
        let mut counter = 0;
        let clock_handle = self.clock.take().unwrap().run();
        let databus_handle = self.databus.take().unwrap().run();
        while !self.cpu.halted {
            counter += 1;
            self.cpu.execute_instruction();
        }

        self.databus = Some(databus_handle.join().unwrap());

        counter
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
        let db = machine.databus.take().unwrap();
        assert_eq!(db.selected_addr, 0x69);
        assert_eq!(db.selected_mode, DatabusMode::Status);
    }

    #[test]
    fn test_write_to_screen() {
        let program = vec!["LoadImm A, 0xf0", "Adr", "LoadImm A, 0x5a", "Write", "Halt"];

        let mut machine = Datapoint::new(program, 1.0);
        machine.run();
        let db = machine.databus.take().unwrap();
        assert_eq!(db.selected_addr, 0xf0);
        assert_eq!(db.screen.buffer[0][0], 'Z');
    }
}
