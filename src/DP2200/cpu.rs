use crate::DP2200::instruction::*;
use log::{error, info, trace};
use std::sync::mpsc::Receiver;

use super::databus::{Databus, DatabusMode};

#[derive(Debug)]
pub struct Cpu {
    pub halted: bool,
    pub intr_enabled: bool,
    pub intr_saved: bool,
    pub memory: [u8; 8192],
    pub alpha_mode: bool,
    pub alpha_registers: [u8; 7],
    pub alpha_flipflops: [bool; 4],
    pub beta_registers: [u8; 7],
    pub beta_flipflops: [bool; 4],
    pub program_counter: u16,
    pub instruction_register: Instruction,
    pub stack: Vec<u16>,
}

impl Cpu {
    pub fn build() -> Cpu {
        Cpu {
            halted: false,
            intr_enabled: false,
            intr_saved: false,
            memory: [0; 8192],
            alpha_mode: true,
            alpha_registers: [0, 0, 0, 0, 0, 0, 0],
            alpha_flipflops: [false, false, false, false],
            beta_registers: [0, 0, 0, 0, 0, 0, 0],
            beta_flipflops: [false, false, false, false],
            program_counter: 0,
            stack: Vec::new(),
            instruction_register: Instruction::unknown(),
        }
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
        if index == 7 {
            self.memory[self.get_hl_address() as usize] = value;
        }

        if self.alpha_mode {
            self.alpha_registers[index as usize] = value;
        } else {
            self.beta_registers[index as usize] = value;
        }
    }

    fn read_reg(&self, index: u8) -> u8 {
        if index == 7 {
            return self.memory[self.get_hl_address() as usize];
        }
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

    fn pop_stack(&mut self) -> Option<u16> {
        self.stack.pop()
    }

    pub fn interrupt(&mut self) {
        self.intr_saved = true;
    }

    pub fn fetch_instruction(&mut self) -> Option<Instruction> {
        let opcode = self.get_from_mem()?;
        let mut inst = Instruction {
            instruction_type: InstructionType::Unknown,
            opcode,
            operand: None,
            address: None,
        };

        let (inst_type, operand, address) = match (
            inst.get_instruction_type(),
            inst.get_destination(),
            inst.get_source(),
        ) {
            (0, _, 6) => (InstructionType::LoadImm, self.get_from_mem(), None),
            (3, 0, 0) => (InstructionType::Nop, None, None),
            (3, 7, 7) => (InstructionType::Halt, None, None),
            (3, _, _) => (InstructionType::Load, None, None),
            (0, 0, 4) => (InstructionType::AddImm, self.get_from_mem(), None),
            (2, 0, _) => (InstructionType::Add, None, None),
            (0, 1, 4) => (InstructionType::AddImmCarry, self.get_from_mem(), None),
            (2, 1, _) => (InstructionType::AddCarry, None, None),
            (0, 2, 4) => (InstructionType::SubImm, self.get_from_mem(), None),
            (2, 2, _) => (InstructionType::Sub, None, None),
            (0, 3, 4) => (InstructionType::SubImmBorrow, self.get_from_mem(), None),
            (2, 3, _) => (InstructionType::SubBorrow, None, None),
            (0, 4, 4) => (InstructionType::AndImm, self.get_from_mem(), None),
            (2, 4, _) => (InstructionType::And, None, None),
            (0, 6, 4) => (InstructionType::OrImm, self.get_from_mem(), None),
            (2, 6, _) => (InstructionType::Or, None, None),
            (0, 5, 4) => (InstructionType::XorImm, self.get_from_mem(), None),
            (2, 5, _) => (InstructionType::Xor, None, None),
            (0, 7, 4) => (InstructionType::CompImm, self.get_from_mem(), None),
            (2, 7, _) => (InstructionType::Comp, None, None),
            (1, 0, 4) => (InstructionType::Jump, None, self.get_16bit_from_mem()),
            (1, c, 0) if c >= 4 => (InstructionType::JumpIf, None, self.get_16bit_from_mem()),
            (1, _, 0) => (InstructionType::JumpIfNot, None, self.get_16bit_from_mem()),
            (1, 0, 6) => (InstructionType::Call, None, self.get_16bit_from_mem()),
            (1, c, 2) if c >= 4 => (InstructionType::CallIf, None, self.get_16bit_from_mem()),
            (1, _, 2) => (InstructionType::CallIfNot, None, self.get_16bit_from_mem()),
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
            (_, _, _) => match inst.opcode {
                0o121 => (InstructionType::Adr, None, None),
                0o123 => (InstructionType::Status, None, None),
                0o125 => (InstructionType::Data, None, None),
                0o127 => (InstructionType::Write, None, None),
                0o131 => (InstructionType::Com1, None, None),
                0o133 => (InstructionType::Com2, None, None),
                0o135 => (InstructionType::Com3, None, None),
                0o137 => (InstructionType::Com4, None, None),
                0o151 => (InstructionType::Beep, None, None),
                0o153 => (InstructionType::Click, None, None),
                0o155 => (InstructionType::Deck1, None, None),
                0o157 => (InstructionType::Deck2, None, None),
                0o161 => (InstructionType::Rbk, None, None),
                0o163 => (InstructionType::Wbk, None, None),
                0o167 => (InstructionType::Bsp, None, None),
                0o171 => (InstructionType::Sf, None, None),
                0o173 => (InstructionType::Sb, None, None),
                0o175 => (InstructionType::Rewind, None, None),
                0o177 => (InstructionType::Tstop, None, None),
                _ => (InstructionType::Unknown, None, None),
            },
        };
        inst.address = address;
        inst.instruction_type = inst_type;
        inst.operand = operand;

        Some(inst)
    }

    pub fn execute_instruction(&mut self, databus: &mut Databus) {
        if self.halted {
            return;
        }

        let inst = self.instruction_register;
        let hl = self.get_hl_address();
        let s = inst.get_source();
        let d = inst.get_destination();
        let c = if d >= 4 { d - 4 } else { d };
        // info!(
        //     "{:#04x}: {:?}",
        //     self.program_counter, self.instruction_register
        // );
        match inst.instruction_type {
            InstructionType::LoadImm => {
                self.write_reg(d, inst.operand.unwrap());
            }
            InstructionType::Load => {
                if d == 7 && s == 7 {
                    self.halted = true;
                } else if d == 7 {
                    if hl as usize >= self.memory.len() {
                    } else {
                        self.memory[hl as usize] = self.read_reg(s);
                    }
                } else if s == 7 {
                    if hl as usize >= self.memory.len() {
                    } else {
                        self.write_reg(d, self.memory[hl as usize]);
                    }
                } else {
                    self.write_reg(d, self.read_reg(s));
                }
            }
            InstructionType::AddImm => {
                let res: u16 = (self.read_reg(0) as u16) + (inst.operand.unwrap() as u16);
                self.write_flag(0, res > 0xff);
                self.write_reg(0, (res & 0xff) as u8);
                self.update_flags();
            }
            InstructionType::Add => {
                let res: u16 = (self.read_reg(0) as u16) + (self.read_reg(s) as u16);
                self.write_flag(0, res > 0xff);
                self.write_reg(0, (res & 0xff) as u8);
                self.update_flags();
            }
            InstructionType::AddImmCarry => {
                let mut res: u16 = (self.read_reg(0) as u16) + (inst.operand.unwrap() as u16);
                if self.read_flag(0) {
                    res += 1;
                }
                self.write_flag(0, res > 0xff);
                self.write_reg(0, (res & 0xff) as u8);
                self.update_flags();
            }
            InstructionType::AddCarry => {
                let mut res: u16 = (self.read_reg(0) as u16) + (self.read_reg(s) as u16);
                if self.read_flag(0) {
                    res += 1;
                }
                self.write_flag(0, res > 0xff);
                self.write_reg(0, (res & 0xff) as u8);
                self.update_flags();
            }
            InstructionType::SubImm => {
                let res: i16 = (self.read_reg(0) as i16) - (inst.operand.unwrap() as i16);
                self.write_flag(0, res < 0);
                self.write_reg(0, res as u8);
                self.update_flags();
            }
            InstructionType::Sub => {
                let res: i16 = (self.read_reg(0) as i16) - (self.read_reg(s) as i16);
                self.write_flag(0, res < 0);
                self.write_reg(0, res as u8);
                self.update_flags();
            }
            InstructionType::SubImmBorrow => {
                let mut res: i16 = (self.read_reg(0) as i16) - (inst.operand.unwrap() as i16);
                if self.read_flag(0) {
                    res -= 1;
                }
                self.write_flag(0, res < 0);
                self.write_reg(0, res as u8);
                self.update_flags();
            }
            InstructionType::SubBorrow => {
                let mut res: i16 = (self.read_reg(0) as i16) - (self.read_reg(s) as i16);
                if self.read_flag(0) {
                    res -= 1;
                }
                self.write_flag(0, res < 0);
                self.write_reg(0, res as u8);
                self.update_flags();
            }
            InstructionType::AndImm => {
                self.write_reg(0, self.read_reg(0) & inst.operand.unwrap());

                self.write_flag(0, false);
                self.update_flags();
            }
            InstructionType::And => {
                self.write_reg(0, self.read_reg(0) & self.read_reg(s));

                self.write_flag(0, false);
                self.update_flags();
            }
            InstructionType::OrImm => {
                self.write_reg(0, self.read_reg(0) | inst.operand.unwrap());

                self.write_flag(0, false);
                self.update_flags();
            }
            InstructionType::Or => {
                self.write_reg(0, self.read_reg(0) | self.read_reg(s));

                self.write_flag(0, false);
                self.update_flags();
            }
            InstructionType::XorImm => {
                self.write_reg(0, self.read_reg(0) ^ inst.operand.unwrap());

                self.write_flag(0, false);
                self.update_flags();
            }
            InstructionType::Xor => {
                self.write_reg(0, self.read_reg(0) ^ self.read_reg(s));

                self.write_flag(0, false);
                self.update_flags();
            }
            InstructionType::CompImm => {
                let saved_reg = self.read_reg(0);
                let res: i16 = (self.read_reg(0) as i16) - (inst.operand.unwrap() as i16);
                self.write_flag(0, res < 0);
                self.write_reg(0, res as u8);
                self.update_flags();
                self.write_reg(0, saved_reg);
            }
            InstructionType::Comp => {
                let saved_reg = self.read_reg(0);
                let res: i16 = (self.read_reg(0) as i16) - (self.read_reg(s) as i16);
                self.write_flag(0, res < 0);
                self.write_reg(0, res as u8);
                self.update_flags();
                self.write_reg(0, saved_reg);
            }
            InstructionType::Jump => {
                self.program_counter = 0x1fff & inst.address.unwrap();
            }
            InstructionType::JumpIf => {
                if self.read_flag(c) {
                    self.program_counter = 0x1fff & inst.address.unwrap();
                }
            }
            InstructionType::JumpIfNot => {
                if !self.read_flag(c) {
                    self.program_counter = 0x1fff & inst.address.unwrap();
                }
            }
            InstructionType::Call => {
                self.push_stack(self.program_counter);
                self.program_counter = 0x1fff & inst.address.unwrap();
            }
            InstructionType::CallIf => {
                if self.read_flag(c) {
                    self.push_stack(self.program_counter);
                    self.program_counter = 0x1fff & inst.address.unwrap();
                }
            }
            InstructionType::CallIfNot => {
                if !self.read_flag(c) {
                    self.push_stack(self.program_counter);
                    self.program_counter = 0x1fff & inst.address.unwrap();
                }
            }
            InstructionType::Return => {
                if let Some(addr) = self.pop_stack() {
                    self.program_counter = addr;
                } else {
                    error!(
                        "Tried to pop empty stack. Cpu program counter: {}, instruction: {:?}",
                        self.program_counter, inst
                    );
                }
            }
            InstructionType::ReturnIf => {
                if self.read_flag(c) {
                    if let Some(addr) = self.pop_stack() {
                        self.program_counter = addr;
                    } else {
                        error!(
                            "Tried to pop empty stack. Cpu program counter: {}, instruction: {:?}",
                            self.program_counter, inst
                        );
                    }
                }
            }
            InstructionType::ReturnIfNot => {
                if !self.read_flag(c) {
                    if let Some(addr) = self.pop_stack() {
                        self.program_counter = addr;
                    } else {
                        error!(
                            "Tried to pop empty stack. Cpu program counter: {}, instruction: {:?}",
                            self.program_counter, inst
                        );
                    }
                }
            }
            InstructionType::ShiftRight => {
                self.write_reg(0, self.read_reg(0).rotate_right(1));
                self.write_flag(0, (self.read_reg(0) & 0x80) == 0x80);
            }
            InstructionType::ShiftLeft => {
                self.write_reg(0, self.read_reg(0).rotate_left(1));
                self.write_flag(0, (self.read_reg(0) & 0x1) == 0x1);
            }
            InstructionType::Nop => {}
            InstructionType::Halt => {
                self.halted = true;
                databus.execute_command(inst, self.read_reg(0));
            }
            InstructionType::Pop => {
                if let Some(value) = self.pop_stack() {
                    self.write_reg(5, ((value >> 8) & 0xff) as u8);
                    self.write_reg(6, (value & 0xff) as u8);
                } else {
                    error!(
                        "Tried to pop empty stack. Cpu program counter: {}, instruction: {:?}",
                        self.program_counter, inst
                    );
                }
            }
            InstructionType::Push => {
                let mut value: u16 = self.read_reg(6) as u16;
                value += (self.read_reg(5) as u16) << 8;
                self.push_stack(value);
            }
            InstructionType::EnableIntr => {
                self.intr_enabled = true;
            }
            InstructionType::DisableInts => {
                self.intr_enabled = false;
            }
            InstructionType::SelectAlpha => {
                self.alpha_mode = true;
            }
            InstructionType::SelectBeta => {
                self.alpha_mode = false;
            }
            InstructionType::Unknown => panic!("Unknown instruction"),
            InstructionType::Input => {
                self.write_reg(0, databus.read_bus());
                databus.strobe();
            }
            InstructionType::Adr => {
                databus.execute_command(inst, self.read_reg(0));
            }
            InstructionType::Status => {
                databus.set_mode(DatabusMode::Status);
            }
            InstructionType::Data => {
                databus.set_mode(DatabusMode::Data);
            }
            InstructionType::Write => {
                databus.execute_command(inst, self.read_reg(0));
            }
            InstructionType::Com1 => {
                databus.execute_command(inst, self.read_reg(0));
            }
            InstructionType::Com2 => {
                databus.execute_command(inst, self.read_reg(0));
            }
            InstructionType::Com3 => {
                databus.execute_command(inst, self.read_reg(0));
            }
            InstructionType::Com4 => {
                databus.execute_command(inst, self.read_reg(0));
            }
            InstructionType::Beep => {
                databus.execute_command(inst, self.read_reg(0));
            }
            InstructionType::Click => {
                databus.execute_command(inst, self.read_reg(0));
            }
            InstructionType::Deck1 => {
                databus.execute_command(inst, self.read_reg(0));
            }
            InstructionType::Deck2 => {
                databus.execute_command(inst, self.read_reg(0));
            }
            InstructionType::Rbk => {
                databus.execute_command(inst, self.read_reg(0));
            }
            InstructionType::Wbk => {
                databus.execute_command(inst, self.read_reg(0));
            }
            InstructionType::Bsp => {
                databus.execute_command(inst, self.read_reg(0));
            }
            InstructionType::Sf => {
                databus.execute_command(inst, self.read_reg(0));
            }
            InstructionType::Sb => {
                databus.execute_command(inst, self.read_reg(0));
            }
            InstructionType::Rewind => {
                databus.execute_command(inst, self.read_reg(0));
            }
            InstructionType::Tstop => {
                databus.execute_command(inst, self.read_reg(0));
            }
        };

        if self.intr_saved {
            // If interrupts are enabled, and we did not enable this cycle
            if self.intr_enabled && inst.instruction_type != InstructionType::EnableIntr {
                // Interrupt triggered
                self.push_stack(self.program_counter);
                self.program_counter = 0;
                self.intr_saved = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;
    use crate::DP2200::datapoint::DataPointRunStatus;
    use crate::DP2200::datapoint::Datapoint;

    fn init_logger() {
        let _ = env_logger::builder()
            // Include all events in tests
            .filter_level(log::LevelFilter::max())
            // Ensure events are captured by `cargo test`
            .is_test(true)
            // Ignore errors initializing the logger if tests race to configure it
            .try_init();
    }

    #[test]
    fn test_fetch_add_inst() {
        let program = vec!["Add 2"];
        let mut machine = Datapoint::from_assembler(program, 1.0);
        let inst = machine.cpu.fetch_instruction().unwrap();
        assert_eq!(inst.instruction_type, InstructionType::Add);
        assert_eq!(inst.operand, None);
        assert_eq!(inst.get_source(), 2);
    }

    #[test]
    fn test_load_imm_inst() {
        let mut machine = Datapoint::from_assembler(vec!["LoadImm A, 10"], 1.0);
        machine.run();
        assert_eq!(machine.cpu.alpha_registers[0], 10);
    }

    #[test]
    fn test_load_imm_neg() {
        let mut machine = Datapoint::from_assembler(vec!["LoadImm A, -10"], 1.0);
        machine.run();
        assert_eq!(machine.cpu.alpha_registers[0] as i8, -10);
    }

    #[test]
    fn test_load_imm_max() {
        let mut machine = Datapoint::from_assembler(vec!["LoadImm A, 255"], 1.0);
        machine.run();
        assert_eq!(machine.cpu.alpha_registers[0], 255);
    }

    #[test]
    fn test_load_imm_min() {
        let mut machine = Datapoint::from_assembler(vec!["LoadImm A, -128"], 1.0);
        machine.run();
        assert_eq!(machine.cpu.alpha_registers[0] as i8, -128);
    }

    #[test]
    fn test_load_imm_zero() {
        let mut machine = Datapoint::from_assembler(vec!["LoadImm A, 0"], 1.0);
        machine.run();
        assert_eq!(machine.cpu.alpha_registers[0], 0);
    }

    #[test]
    fn test_load_reg_to_reg_inst() {
        let mut machine = Datapoint::from_assembler(vec!["Load B, A", "Halt"], 1.0);
        machine.cpu.alpha_registers[0] = 10;
        machine.run();

        assert_eq!(machine.cpu.alpha_registers[1], 10);
        assert_eq!(machine.cpu.alpha_registers[0], 10);
    }

    #[test]
    fn test_load_reg_to_reg_inst_neg() {
        let mut machine = Datapoint::from_assembler(vec!["Load B, A", "Halt"], 1.0);
        machine.cpu.alpha_registers[0] = (-10_i8 as u8);
        machine.run();

        assert_eq!(machine.cpu.alpha_registers[1] as i8, -10);
        assert_eq!(machine.cpu.alpha_registers[0] as i8, -10);
    }

    #[test]
    fn test_load_mem_to_reg() {
        let mut machine = Datapoint::from_assembler(vec!["Load A, M", "Halt"], 1.0);
        let addr: u16 = 0x1eef;
        machine.cpu.alpha_registers[5] = (addr >> 8) as u8;
        machine.cpu.alpha_registers[6] = (addr & 0xff) as u8;
        machine.cpu.memory[addr as usize] = 10;
        machine.run();

        assert_eq!(machine.cpu.alpha_registers[0], 10);
        assert_eq!(machine.cpu.memory[addr as usize], 10);
    }

    #[test]
    fn test_load_reg_to_mem() {
        let mut machine = Datapoint::from_assembler(vec!["Load M, A", "Halt"], 1.0);
        let addr: u16 = 0x1eef;
        machine.cpu.alpha_registers[5] = (addr >> 8) as u8;
        machine.cpu.alpha_registers[6] = (addr & 0xff) as u8;
        machine.cpu.alpha_registers[0] = 10;
        machine.run();

        assert_eq!(machine.cpu.alpha_registers[0], 10);
        assert_eq!(machine.cpu.memory[addr as usize], 10);
    }

    #[test]
    fn test_load_mem_to_mem() {
        let mut machine = Datapoint::from_assembler(vec!["Load M, M"], 1.0);
        let status = machine.single_step();
        assert_eq!(status, DataPointRunStatus::Halted)
    }

    #[test]
    fn test_add_inst() {
        init_logger();

        let program = vec!["LoadImm A, 10", "AddImm 10", "Halt"];
        let mut machine = Datapoint::from_assembler(program, 1.0);

        machine.run();

        assert_eq!(machine.cpu.read_reg(0), 20);
        assert!(!machine.cpu.read_flag(0));
        assert!(!machine.cpu.read_flag(1));
        assert!(!machine.cpu.read_flag(2));
        assert!(!machine.cpu.read_flag(3));
    }

    #[test]
    fn test_add_odd_parity_inst() {
        let program = vec!["LoadImm A, 10", "AddImm 11", "Halt"];
        let mut machine = Datapoint::from_assembler(program, 1.0);

        machine.run();

        assert_eq!(machine.cpu.read_reg(0), 21);
        assert!(!machine.cpu.read_flag(0));
        assert!(!machine.cpu.read_flag(1));
        assert!(!machine.cpu.read_flag(2));
        assert!(machine.cpu.read_flag(3));
    }

    #[test]
    fn test_add_sign_flag_inst() {
        let program = vec!["LoadImm A, 10", "AddImm 138", "Halt"];
        let mut machine = Datapoint::from_assembler(program, 1.0);

        machine.run();

        assert!(!machine.cpu.read_flag(0));
        assert!(!machine.cpu.read_flag(1));
        assert!(machine.cpu.read_flag(2));
        assert!(machine.cpu.read_flag(3));
    }

    #[test]
    fn test_add_zero_and_overflow_inst() {
        let program = vec!["LoadImm A, 10", "AddImm 246", "Halt"];
        let mut machine = Datapoint::from_assembler(program, 1.0);

        machine.run();

        assert!(machine.cpu.read_flag(0));
        assert!(machine.cpu.read_flag(1));
        assert!(!machine.cpu.read_flag(2));
        assert!(!machine.cpu.read_flag(3));
    }

    #[test]
    fn test_sub_underflow() {
        let program = vec!["LoadImm A, 10", "SubImm 11", "Halt"];

        let mut machine = Datapoint::from_assembler(program, 1.0);
        machine.run();

        assert!(machine.cpu.read_flag(0));
        assert_eq!(machine.cpu.read_reg(0) as i8, -1);
    }

    #[test]
    fn test_call() {
        let program = vec![
            "Nop",        //addr 0
            "Call test",  // addr 1, 2, 3
            "Halt",       // addr 4
            "test: Halt", // addr 5
        ];

        let mut machine = Datapoint::from_assembler(program, 1.0);
        machine.run();

        assert_eq!(machine.cpu.pop_stack().unwrap(), 4);
    }

    #[test]
    fn test_return() {
        let program = vec!["Call test", "Halt", "test: LoadImm B, 10", "Return"];

        let mut machine = Datapoint::from_assembler(program, 1.0);
        machine.run();

        assert_eq!(machine.cpu.read_reg(1), 10);
    }

    #[test]
    fn test_push_stack() {
        let program = vec!["LoadImm H, 0x88", "LoadImm L, 0x77", "Push", "Halt"];

        let mut machine = Datapoint::from_assembler(program, 1.0);
        machine.run();

        assert_eq!(machine.cpu.pop_stack().unwrap(), 0x8877);
    }

    #[test]
    fn test_pop_stack() {
        let program = vec!["Call test", "Nop", "Nop", "Nop", "test: Pop", "Halt"];

        let mut machine = Datapoint::from_assembler(program, 1.0);
        machine.run();

        assert_eq!(machine.cpu.get_hl_address(), 0x3);
    }

    #[test]
    fn test_select_beta() {
        let program = vec![
            "SelectBeta",
            "LoadImm A, 10",
            "SelectAlpha",
            "LoadImm A, 20",
            "SelectBeta",
            "Halt",
        ];

        let mut machine = Datapoint::from_assembler(program, 1.0);
        machine.run();

        assert_eq!(machine.cpu.read_reg(0), 10);
        assert!(!machine.cpu.alpha_mode);
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

        let mut machine = Datapoint::from_assembler(program, 1000.0);

        let counter = machine.run();
        println!("{}", counter);
    }

    #[test]
    fn test_comp_zero() {
        let program = vec!["LoadImm A, 10", "LoadImm B, 10", "Comp B", "Halt"];

        let mut machine = Datapoint::from_assembler(program, 1000.0);
        machine.run();
        assert!(!machine.cpu.read_flag(0));
        assert!(machine.cpu.read_flag(1));
    }
}
