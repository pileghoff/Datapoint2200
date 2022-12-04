use crate::{clock::*, instruction::*};
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
pub struct Cpu {
    pub halted: bool,
    pub intr_enabled: bool,
    pub intr_saved: bool,
    pub memory: [u8; 4096],
    pub alpha_mode: bool,
    pub alpha_registers: [u8; 7],
    pub alpha_flipflops: [bool; 4],
    pub beta_registers: [u8; 7],
    pub beta_flipflops: [bool; 4],
    pub program_counter: u16,
    pub instruction_register: Instruction,
    pub stack: Vec<u16>,
    pub clock: Option<Receiver<u8>>,
    pub intr: Option<Receiver<u8>>,
}

impl Cpu {
    pub fn new(mem: Vec<u8>, clock: Option<Receiver<u8>>, intr: Option<Receiver<u8>>) -> Cpu {
        let mut cpu = Cpu {
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
            instruction_register: Instruction {
                instruction_type: InstructionType::Unknown,
                opcode: 0,
                operand: None,
                address: None,
            },
            stack: Vec::new(),
            clock: clock,
            intr: intr,
        };
        for (i, b) in mem.iter().enumerate() {
            cpu.memory[i] = *b;
        }

        cpu
    }

    fn get_from_mem(&mut self) -> Option<u8> {
        let res = self.memory.get(self.program_counter as usize)?;
        self.program_counter += 1;
        Some(*res)
    }

    fn get_16bit_from_mem(&mut self) -> Option<u16> {
        Some(self.get_from_mem()? as u16 + ((self.get_from_mem()? as u16) << 8))
    }

    fn write_reg(&mut self, index: u8, value: u8) {
        if self.alpha_mode {
            self.alpha_registers[index as usize] = value;
        } else {
            self.beta_registers[index as usize] = value;
        }
    }

    fn read_reg(&self, index: u8) -> u8 {
        if self.alpha_mode {
            self.alpha_registers[index as usize]
        } else {
            self.beta_registers[index as usize]
        }
    }

    fn write_flag(&mut self, index: u8, value: bool) {
        if self.alpha_mode {
            self.alpha_flipflops[index as usize] = value;
        } else {
            self.beta_flipflops[index as usize] = value;
        }
    }

    fn read_flag(&self, index: u8) -> bool {
        if self.alpha_mode {
            self.alpha_flipflops[index as usize]
        } else {
            self.beta_flipflops[index as usize]
        }
    }

    fn update_flags(&mut self) {
        self.write_flag(1, self.read_reg(0) == 0);
        self.write_flag(2, self.read_reg(0) & 0x80 != 0);
        self.write_flag(3, self.read_reg(0).count_ones() % 2 != 0);
    }

    fn get_hl_address(&self) -> u16 {
        ((self.read_reg(5) as u16) << 8) + self.read_reg(6) as u16
    }

    fn push_stack(&mut self, value: u16) {
        self.stack.push(value);
        while self.stack.len() >= 16 {
            self.stack.remove(0);
        }
    }

    fn pop_stack(&mut self) -> u16 {
        self.stack.pop().unwrap()
    }
}

pub fn execute_instruction(cpu: &mut Cpu) {
    let inst = &cpu.instruction_register.clone();
    let hl = cpu.get_hl_address();
    let s = inst.get_source();
    let d = inst.get_destination();
    let c = if d >= 4 { d - 4 } else { d };
    match inst.instruction_type {
        InstructionType::LoadImm => {
            cpu.write_reg(d, inst.operand.unwrap());
        }
        InstructionType::Load => {
            if d == 7 && s == 7 {
                cpu.halted = true;
            } else if d == 7 {
                cpu.memory[hl as usize] = cpu.read_reg(s);
            } else if s == 7 {
                cpu.write_reg(d, cpu.memory[hl as usize]);
            } else {
                cpu.write_reg(d, cpu.read_reg(s));
            }
        }
        InstructionType::AddImm => {
            let res: u16 = (cpu.read_reg(0) as u16) + (inst.operand.unwrap() as u16);
            cpu.write_flag(0, res > 0xff);
            cpu.write_reg(0, (res & 0xff) as u8);
            cpu.update_flags();
        }
        InstructionType::Add => {
            let res: u16 = (cpu.read_reg(0) as u16) + (cpu.read_reg(s) as u16);
            cpu.write_flag(0, res > 0xff);
            cpu.write_reg(0, (res & 0xff) as u8);
            cpu.update_flags();
        }
        InstructionType::AddImmCarry => {
            let mut res: u16 = (cpu.read_reg(0) as u16) + (inst.operand.unwrap() as u16);
            if cpu.read_flag(0) {
                res += 1;
            }
            cpu.write_flag(0, res > 0xff);
            cpu.write_reg(0, (res & 0xff) as u8);
            cpu.update_flags();
        }
        InstructionType::AddCarry => {
            let mut res: u16 = (cpu.read_reg(0) as u16) + (cpu.read_reg(s) as u16);
            if cpu.read_flag(0) {
                res += 1;
            }
            cpu.write_flag(0, res > 0xff);
            cpu.write_reg(0, (res & 0xff) as u8);
            cpu.update_flags();
        }
        InstructionType::SubImm => {
            let res: i16 = (cpu.read_reg(0) as i16) - (inst.operand.unwrap() as i16);
            cpu.write_flag(0, res < 0);
            cpu.write_reg(0, res as u8);
            cpu.update_flags();
        }
        InstructionType::Sub => {
            let res: i16 = (cpu.read_reg(0) as i16) - (cpu.read_reg(s) as i16);
            cpu.write_flag(0, res < 0);
            cpu.write_reg(0, res as u8);
            cpu.update_flags();
        }
        InstructionType::SubImmBorrow => {
            let mut res: i16 = (cpu.read_reg(0) as i16) - (inst.operand.unwrap() as i16);
            if cpu.read_flag(0) {
                res -= 1;
            }
            cpu.write_flag(0, res < 0);
            cpu.write_reg(0, res as u8);
            cpu.update_flags();
        }
        InstructionType::SubBorrow => {
            let mut res: i16 = (cpu.read_reg(0) as i16) - (cpu.read_reg(s) as i16);
            if cpu.read_flag(0) {
                res -= 1;
            }
            cpu.write_flag(0, res < 0);
            cpu.write_reg(0, res as u8);
            cpu.update_flags();
        }
        InstructionType::AndImm => {
            cpu.write_reg(0, cpu.read_reg(0) & inst.operand.unwrap());

            cpu.write_flag(0, false);
            cpu.update_flags();
        }
        InstructionType::And => {
            cpu.write_reg(0, cpu.read_reg(0) & cpu.read_reg(s));

            cpu.write_flag(0, false);
            cpu.update_flags();
        }
        InstructionType::OrImm => {
            cpu.write_reg(0, cpu.read_reg(0) | inst.operand.unwrap());

            cpu.write_flag(0, false);
            cpu.update_flags();
        }
        InstructionType::Or => {
            cpu.write_reg(0, cpu.read_reg(0) | inst.operand.unwrap());

            cpu.write_flag(0, false);
            cpu.update_flags();
        }
        InstructionType::XorImm => {
            cpu.write_reg(0, cpu.read_reg(0) ^ inst.operand.unwrap());

            cpu.write_flag(0, false);
            cpu.update_flags();
        }
        InstructionType::Xor => {
            cpu.write_reg(0, cpu.read_reg(0) ^ inst.operand.unwrap());

            cpu.write_flag(0, false);
            cpu.update_flags();
        }
        InstructionType::CompImm => {
            let res: i16 = (cpu.read_reg(0) as i16) - (inst.operand.unwrap() as i16);
            cpu.write_flag(0, res < 0);
            cpu.update_flags();
        }
        InstructionType::Comp => {
            let res: i16 = (cpu.read_reg(0) as i16) - (cpu.read_reg(s) as i16);
            cpu.write_flag(0, res < 0);
            cpu.update_flags();
        }
        InstructionType::Jump => {
            cpu.program_counter = 0x1fff & inst.address.unwrap();
        }
        InstructionType::JumpIf => {
            if cpu.read_flag(c) {
                cpu.program_counter = 0x1fff & inst.address.unwrap();
            }
        }
        InstructionType::JumpIfNot => {
            if !cpu.read_flag(c) {
                cpu.program_counter = 0x1fff & inst.address.unwrap();
            }
        }
        InstructionType::Call => {
            cpu.push_stack(cpu.program_counter);
            cpu.program_counter = 0x1fff & inst.address.unwrap();
        }
        InstructionType::CallIf => {
            if cpu.read_flag(c) {
                cpu.push_stack(cpu.program_counter);
                cpu.program_counter = 0x1fff & inst.address.unwrap();
            }
        }
        InstructionType::CallIfNot => {
            if !cpu.read_flag(c) {
                cpu.push_stack(cpu.program_counter);
                cpu.program_counter = 0x1fff & inst.address.unwrap();
            }
        }
        InstructionType::Return => {
            cpu.program_counter = cpu.pop_stack();
        }
        InstructionType::ReturnIf => {
            if cpu.read_flag(c) {
                cpu.program_counter = cpu.pop_stack();
            }
        }
        InstructionType::ReturnIfNot => {
            if !cpu.read_flag(c) {
                cpu.program_counter = cpu.pop_stack();
            }
        }
        InstructionType::ShiftRight => {
            cpu.write_reg(0, cpu.read_reg(0).rotate_right(1));
            cpu.write_flag(0, (cpu.read_reg(0) & 0x80) == 0x80);
        }
        InstructionType::ShiftLeft => {
            cpu.write_reg(0, cpu.read_reg(0).rotate_left(1));
            cpu.write_flag(0, (cpu.read_reg(0) & 0x1) == 0x1);
        }
        InstructionType::Nop => {}
        InstructionType::Halt => cpu.halted = true,
        InstructionType::Pop => {
            let value = cpu.pop_stack();
            cpu.write_reg(5, ((value >> 8) & 0xff) as u8);
            cpu.write_reg(6, (value & 0xff) as u8);
        }
        InstructionType::Push => {
            let mut value: u16 = cpu.read_reg(6) as u16;
            value += (cpu.read_reg(5) as u16) << 8;
            cpu.push_stack(value);
        }
        InstructionType::EnableIntr => {
            cpu.intr_enabled = true;
        }
        InstructionType::DisableInts => {
            cpu.intr_enabled = false;
        }
        InstructionType::SelectAlpha => {
            cpu.alpha_mode = true;
        }
        InstructionType::SelectBeta => {
            cpu.alpha_mode = false;
        }
        InstructionType::Unknown => panic!("Unknown instruction"),
        InstructionType::Input => unimplemented!(),
        InstructionType::Adr => todo!(),
        InstructionType::Status => todo!(),
        InstructionType::Data => todo!(),
        InstructionType::Write => todo!(),
        InstructionType::Com1 => todo!(),
        InstructionType::Com2 => todo!(),
        InstructionType::Com3 => todo!(),
        InstructionType::Com4 => todo!(),
        InstructionType::Beep => todo!(),
        InstructionType::Click => todo!(),
        InstructionType::Deck1 => todo!(),
        InstructionType::Deck2 => todo!(),
        InstructionType::Rbk => todo!(),
        InstructionType::Wbk => todo!(),
        InstructionType::Bsp => todo!(),
        InstructionType::Sf => todo!(),
        InstructionType::Sb => todo!(),
        InstructionType::Rewind => todo!(),
        InstructionType::Tstop => todo!(),
    };

    if let Some(clock) = &cpu.clock {
        for i in 0..inst.get_clock_cycles() {
            clock.recv().unwrap();
        }
    }

    if let Some(intr) = &cpu.intr {
        let intr_happened = intr.try_recv();
        if intr_happened.is_ok() {
            cpu.intr_saved = true;
        };
    }

    if cpu.intr_saved {
        // If interrupts are enabled, and we did not enable this cycle
        if cpu.intr_enabled && inst.instruction_type != InstructionType::EnableIntr {
            // Interrupt triggered
            cpu.push_stack(cpu.program_counter);
            cpu.program_counter = 0;
            cpu.intr_saved = false;
        }
    }
}

pub fn fetch_instruction(cpu: &mut Cpu) {
    let opcode = cpu.get_from_mem();
    if opcode.is_none() {
        cpu.instruction_register = Instruction {
            instruction_type: InstructionType::Unknown,
            opcode: 0,
            operand: None,
            address: None,
        };
        return;
    }
    let mut inst = Instruction {
        instruction_type: InstructionType::Unknown,
        opcode: opcode.unwrap(),
        operand: None,
        address: None,
    };

    let (inst_type, operand, address) = match (
        inst.get_instruction_type(),
        inst.get_destination(),
        inst.get_source(),
    ) {
        (0, _, 6) => (InstructionType::LoadImm, cpu.get_from_mem(), None),
        (3, 0, 0) => (InstructionType::Nop, None, None),
        (3, 7, 7) => (InstructionType::Halt, None, None),
        (3, _, _) => (InstructionType::Load, None, None),
        (0, 0, 4) => (InstructionType::AddImm, cpu.get_from_mem(), None),
        (2, 0, _) => (InstructionType::Add, None, None),
        (0, 1, 4) => (InstructionType::AddImmCarry, cpu.get_from_mem(), None),
        (2, 1, _) => (InstructionType::AddCarry, None, None),
        (0, 2, 4) => (InstructionType::SubImm, cpu.get_from_mem(), None),
        (2, 2, _) => (InstructionType::Sub, None, None),
        (0, 3, 4) => (InstructionType::SubImmBorrow, cpu.get_from_mem(), None),
        (2, 3, _) => (InstructionType::SubBorrow, None, None),
        (0, 4, 4) => (InstructionType::AndImm, cpu.get_from_mem(), None),
        (2, 4, _) => (InstructionType::And, None, None),
        (0, 6, 4) => (InstructionType::OrImm, cpu.get_from_mem(), None),
        (2, 6, _) => (InstructionType::Or, None, None),
        (0, 5, 4) => (InstructionType::XorImm, cpu.get_from_mem(), None),
        (2, 5, _) => (InstructionType::Xor, None, None),
        (0, 7, 4) => (InstructionType::CompImm, cpu.get_from_mem(), None),
        (2, 7, _) => (InstructionType::Comp, None, None),
        (1, 0, 4) => (InstructionType::Jump, None, cpu.get_16bit_from_mem()),
        (1, c, 0) if c >= 4 => (InstructionType::JumpIf, None, cpu.get_16bit_from_mem()),
        (1, _, 0) => (InstructionType::JumpIfNot, None, cpu.get_16bit_from_mem()),
        (1, 0, 6) => (InstructionType::Call, None, cpu.get_16bit_from_mem()),
        (1, c, 2) if c >= 4 => (InstructionType::CallIf, None, cpu.get_16bit_from_mem()),
        (1, _, 2) => (InstructionType::CallIfNot, None, cpu.get_16bit_from_mem()),
        (0, 0, 7) => (InstructionType::Return, None, None),
        (0, c, 3) if c >= 4 => (InstructionType::ReturnIf, None, None),
        (0, _, 3) => (InstructionType::ReturnIfNot, None, None),
        (0, 1, 2) => (InstructionType::ShiftRight, None, None),
        (0, 0, 2) => (InstructionType::ShiftLeft, None, None),
        (0, 0, 0) => (InstructionType::Halt, None, None),
        (0, 0, 1) => (InstructionType::Halt, None, None),
        (1, 0, 1) => (InstructionType::Input, None, None),
        (0, 6, 0) => (InstructionType::Pop, None, None),
        (0, 7, 0) => (InstructionType::Push, None, None),
        (0, 5, 0) => (InstructionType::EnableIntr, None, None),
        (0, 4, 0) => (InstructionType::DisableInts, None, None),
        (0, 3, 0) => (InstructionType::SelectAlpha, None, None),
        (0, 2, 0) => (InstructionType::SelectBeta, None, None),
        (_, _, _) => panic!("Unknown instruction"),
    };
    inst.address = address;
    inst.instruction_type = inst_type;
    inst.operand = operand;

    cpu.instruction_register = inst;
}

fn run_to_halt(cpu: &mut Cpu) -> u32 {
    let mut counter = 0;
    while !cpu.halted {
        counter += 1;
        fetch_instruction(cpu);
        execute_instruction(cpu);
    }

    counter
}

#[cfg(test)]
mod tests {
    use std::time;

    use super::*;
    use crate::{assembler::assemble, datapoint::Datapoint};

    #[test]
    fn test_fetch_add_inst() {
        let program = assemble(vec!["Add 2"]);
        let mut cpu = Cpu::new(program, None, None);
        fetch_instruction(&mut cpu);
        assert_eq!(
            cpu.instruction_register.instruction_type,
            InstructionType::Add
        );
        assert_eq!(cpu.instruction_register.operand, None);
        assert_eq!(cpu.instruction_register.get_source(), 2);
    }

    #[test]
    fn test_load_imm_inst() {
        let program = assemble(vec!["LoadImm A, 10"]);
        let mut cpu = Cpu::new(program, None, None);
        fetch_instruction(&mut cpu);
        execute_instruction(&mut cpu);
        assert_eq!(cpu.alpha_registers[0], 10);
    }

    #[test]
    fn test_load_reg_to_reg_inst() {
        let program = assemble(vec!["LoadImm A, 10", "Load B, A", "Halt"]);
        let mut cpu = Cpu::new(program, None, None);

        run_to_halt(&mut cpu);

        assert_eq!(cpu.alpha_registers[1], 10);
    }

    #[test]
    fn test_add_inst() {
        let program = assemble(vec!["LoadImm A, 10", "AddImm 10", "Halt"]);
        let mut cpu = Cpu::new(program, None, None);

        run_to_halt(&mut cpu);

        assert_eq!(cpu.read_reg(0), 20);
        assert_eq!(cpu.read_flag(0), false);
        assert_eq!(cpu.read_flag(1), false);
        assert_eq!(cpu.read_flag(2), false);
        assert_eq!(cpu.read_flag(3), false);
    }

    #[test]
    fn test_add_odd_parity_inst() {
        let program = assemble(vec!["LoadImm A, 10", "AddImm 11", "Halt"]);
        let mut cpu = Cpu::new(program, None, None);

        run_to_halt(&mut cpu);

        assert_eq!(cpu.read_reg(0), 21);
        assert_eq!(cpu.read_flag(0), false);
        assert_eq!(cpu.read_flag(1), false);
        assert_eq!(cpu.read_flag(2), false);
        assert_eq!(cpu.read_flag(3), true);
    }

    #[test]
    fn test_add_sign_flag_inst() {
        let program = assemble(vec!["LoadImm A, 10", "AddImm 138", "Halt"]);
        let mut cpu = Cpu::new(program, None, None);

        run_to_halt(&mut cpu);

        assert_eq!(cpu.read_flag(0), false);
        assert_eq!(cpu.read_flag(1), false);
        assert_eq!(cpu.read_flag(2), true);
        assert_eq!(cpu.read_flag(3), true);
    }

    #[test]
    fn test_add_zero_and_overflow_inst() {
        let program = assemble(vec!["LoadImm A, 10", "AddImm 246", "Halt"]);
        let mut cpu = Cpu::new(program, None, None);

        run_to_halt(&mut cpu);

        assert_eq!(cpu.read_flag(0), true);
        assert_eq!(cpu.read_flag(1), true);
        assert_eq!(cpu.read_flag(2), false);
        assert_eq!(cpu.read_flag(3), false);
    }

    #[test]
    fn test_sub_underflow() {
        let program = assemble(vec!["LoadImm A, 10", "SubImm 11", "Halt"]);

        let mut cpu = Cpu::new(program, None, None);
        run_to_halt(&mut cpu);

        assert_eq!(cpu.read_flag(0), true);
        assert_eq!(cpu.read_reg(0) as i8, -1);
    }

    #[test]
    fn test_jump_if_not() {
        let program = assemble(vec![
            //                                     Number of times instruction is run
            "LoadImm A, 10",     // 10                       =  1
            "loop: SubImm 1",    // 9, 8,7,6,5,4,3,2,1,0,-1  = 11
            "JumpIfNot 0, loop", //9,8,7,6,5,4,3,2,1,0,-1    = 11
            "Halt",              // -1                       =  1
        ]); //                                                 24 total

        let mut cpu = Cpu::new(program, None, None);
        let insts_cnt = run_to_halt(&mut cpu);
        assert_eq!(insts_cnt, 24);
    }

    #[test]
    fn test_call() {
        let program = assemble(vec![
            "Nop",        //addr 0
            "Call test",  // addr 1, 2, 3
            "Halt",       // addr 4
            "test: Halt", // addr 5
        ]);

        let mut cpu = Cpu::new(program, None, None);
        run_to_halt(&mut cpu);

        assert_eq!(cpu.pop_stack(), 4);
        assert_eq!(cpu.program_counter, 6);
    }

    #[test]
    fn test_return() {
        let program = assemble(vec!["Call test", "Halt", "test: LoadImm B, 10", "Return"]);

        let mut cpu = Cpu::new(program, None, None);
        run_to_halt(&mut cpu);

        assert_eq!(cpu.read_reg(1), 10);
        assert_eq!(cpu.program_counter, 4);
    }

    #[test]
    fn test_push_stack() {
        let program = assemble(vec!["LoadImm H, 0x88", "LoadImm L, 0x77", "Push", "Halt"]);

        let mut cpu = Cpu::new(program, None, None);
        run_to_halt(&mut cpu);

        assert_eq!(cpu.pop_stack(), 0x8877);
    }

    #[test]
    fn test_pop_stack() {
        let program = assemble(vec!["Call test", "Nop", "Nop", "Nop", "test: Pop", "Halt"]);

        let mut cpu = Cpu::new(program, None, None);
        run_to_halt(&mut cpu);

        assert_eq!(cpu.get_hl_address(), 0x3);
    }

    #[test]
    fn test_select_beta() {
        let program = assemble(vec![
            "SelectBeta",
            "LoadImm A, 10",
            "SelectAlpha",
            "LoadImm A, 20",
            "SelectBeta",
            "Halt",
        ]);

        let mut cpu = Cpu::new(program, None, None);
        run_to_halt(&mut cpu);

        assert_eq!(cpu.read_reg(0), 10);
        assert_eq!(cpu.alpha_mode, false);
    }

    #[test]
    fn test_clock() {
        let program = vec![
            "SelectBeta",    // 2
            "LoadImm A, 10", // 2
            "SelectAlpha",   // 2
            "LoadImm A, 20", // 2
            "SelectBeta",    // 2
            "Halt",          // 0
        ];
        let mut machine = Datapoint::new(program, 1000.0);
        let start = time::Instant::now();
        machine.run();
        let elapsed = start.elapsed();
        // 10 cycles, at 1000 times slowdown should take = 16000 us
        print!("{}", elapsed.as_micros());
        assert!(elapsed.as_micros() > 16000);
        assert!(elapsed.as_micros() < 18000);
    }

    #[test]
    fn test_intr() {
        let program = vec![
            "CompImm 1",      // Carry flag gets set if A is 0 (First time)
            "JumpIf Cf, run", // Jump if A was 0
            "Halt",           // Only halt, only get here if the interrupts actually works
            "run: LoadImm A, 1",
            "EnableIntr",
            "Nop",
            "Jump run",
        ];

        let mut machine = Datapoint::new(program, 1000.0);

        let counter = machine.run();
        println!("{}", counter);
    }
}
