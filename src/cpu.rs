use crate::instruction::*;

#[derive(Debug, Clone)]
pub struct Cpu {
    pub halted: bool,
    pub memory: [u8; 4096],
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
    pub fn new(mem: Vec<u8>) -> Cpu {
        let mut cpu = Cpu {
            halted: false,
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
        };
        for (i, b) in mem.iter().enumerate() {
            cpu.memory[i] = *b;
        }

        cpu
    }

    fn get_from_mem(&mut self) -> u8 {
        let res = self.memory.get(self.program_counter as usize).unwrap();
        self.program_counter += 1;
        *res
    }

    fn get_16bit_from_mem(&mut self) -> u16 {
        self.get_from_mem() as u16 + ((self.get_from_mem() as u16) << 8)
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
}

fn execute_instruction(cpu: &mut Cpu) {
    let inst = &cpu.instruction_register;
    let hl = cpu.get_hl_address();
    let s = inst.get_source();
    let d = inst.get_destination();
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
            if cpu.read_flag(d) {
                cpu.program_counter = 0x1fff & inst.address.unwrap();
            }
        }
        InstructionType::JumpIfNot => {
            if !cpu.read_flag(d) {
                cpu.program_counter = 0x1fff & inst.address.unwrap();
            }
        }
        InstructionType::Call => unimplemented!(),
        InstructionType::CallIf => unimplemented!(),
        InstructionType::CallIfNot => unimplemented!(),
        InstructionType::Return => unimplemented!(),
        InstructionType::ReturnIf => unimplemented!(),
        InstructionType::ReturnIfNot => unimplemented!(),
        InstructionType::ShiftRight => unimplemented!(),
        InstructionType::ShiftLeft => unimplemented!(),
        InstructionType::Nop => unimplemented!(),
        InstructionType::Halt => cpu.halted = true,
        InstructionType::Input => unimplemented!(),
        InstructionType::Pop => unimplemented!(),
        InstructionType::Push => unimplemented!(),
        InstructionType::EnableIntr => unimplemented!(),
        InstructionType::DisableInts => unimplemented!(),
        InstructionType::SelectAlpha => unimplemented!(),
        InstructionType::SelectBeta => unimplemented!(),
        InstructionType::Unknown => panic!("Unknown instruction"),
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
}

pub fn fetch_instruction(cpu: &mut Cpu) {
    let mut inst = Instruction {
        instruction_type: InstructionType::Unknown,
        opcode: cpu.get_from_mem(),
        operand: None,
        address: None,
    };

    let (inst_type, operand, address) = match (
        inst.get_instruction_type(),
        inst.get_destination(),
        inst.get_source(),
    ) {
        (0, _, 6) => (InstructionType::LoadImm, Some(cpu.get_from_mem()), None),
        (3, 0, 0) => (InstructionType::Nop, None, None),
        (3, 7, 7) => (InstructionType::Halt, None, None),
        (3, _, _) => (InstructionType::Load, None, None),
        (0, 0, 4) => (InstructionType::AddImm, Some(cpu.get_from_mem()), None),
        (2, 0, _) => (InstructionType::Add, None, None),
        (0, 1, 4) => (InstructionType::AddImmCarry, Some(cpu.get_from_mem()), None),
        (2, 1, _) => (InstructionType::AddCarry, None, None),
        (0, 2, 4) => (InstructionType::SubImm, Some(cpu.get_from_mem()), None),
        (2, 2, _) => (InstructionType::Sub, None, None),
        (0, 3, 4) => (
            InstructionType::SubImmBorrow,
            Some(cpu.get_from_mem()),
            None,
        ),
        (2, 3, _) => (InstructionType::SubBorrow, None, None),
        (0, 4, 4) => (InstructionType::AndImm, Some(cpu.get_from_mem()), None),
        (2, 4, _) => (InstructionType::And, None, None),
        (0, 6, 4) => (InstructionType::OrImm, Some(cpu.get_from_mem()), None),
        (2, 6, _) => (InstructionType::Or, None, None),
        (0, 5, 4) => (InstructionType::XorImm, Some(cpu.get_from_mem()), None),
        (2, 5, _) => (InstructionType::Xor, None, None),
        (0, 7, 4) => (InstructionType::CompImm, Some(cpu.get_from_mem()), None),
        (2, 7, _) => (InstructionType::Comp, None, None),
        (1, 0, 4) => (InstructionType::Jump, None, Some(cpu.get_16bit_from_mem())),
        (1, c, 0) if c >= 4 => (
            InstructionType::JumpIf,
            None,
            Some(cpu.get_16bit_from_mem()),
        ),
        (1, _, 0) => (
            InstructionType::JumpIfNot,
            None,
            Some(cpu.get_16bit_from_mem()),
        ),
        (1, 0, 6) => (InstructionType::Call, None, Some(cpu.get_16bit_from_mem())),
        (1, c, 2) if c >= 4 => (
            InstructionType::CallIf,
            None,
            Some(cpu.get_16bit_from_mem()),
        ),
        (1, _, 2) => (
            InstructionType::CallIfNot,
            None,
            Some(cpu.get_16bit_from_mem()),
        ),
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
    use super::*;
    use crate::assembler::assemble;

    #[test]
    fn test_fetch_add_inst() {
        let program = assemble(vec!["Add 2"]);
        let mut cpu = Cpu::new(program);
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
        let mut cpu = Cpu::new(program);
        fetch_instruction(&mut cpu);
        execute_instruction(&mut cpu);
        assert_eq!(cpu.alpha_registers[0], 10);
    }

    #[test]
    fn test_load_reg_to_reg_inst() {
        let program = assemble(vec!["LoadImm A, 10", "Load B, A", "Halt"]);
        let mut cpu = Cpu::new(program);

        run_to_halt(&mut cpu);

        assert_eq!(cpu.alpha_registers[1], 10);
    }

    #[test]
    fn test_add_inst() {
        let program = assemble(vec!["LoadImm A, 10", "AddImm 10", "Halt"]);
        let mut cpu = Cpu::new(program);

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
        let mut cpu = Cpu::new(program);

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
        let mut cpu = Cpu::new(program);

        run_to_halt(&mut cpu);

        assert_eq!(cpu.read_flag(0), false);
        assert_eq!(cpu.read_flag(1), false);
        assert_eq!(cpu.read_flag(2), true);
        assert_eq!(cpu.read_flag(3), true);
    }

    #[test]
    fn test_add_zero_and_overflow_inst() {
        let program = assemble(vec!["LoadImm A, 10", "AddImm 246", "Halt"]);
        let mut cpu = Cpu::new(program);

        run_to_halt(&mut cpu);

        assert_eq!(cpu.read_flag(0), true);
        assert_eq!(cpu.read_flag(1), true);
        assert_eq!(cpu.read_flag(2), false);
        assert_eq!(cpu.read_flag(3), false);
    }

    #[test]
    fn test_sub_underflow() {
        let program = assemble(vec!["LoadImm A, 10", "SubImm 11", "Halt"]);

        let mut cpu = Cpu::new(program);
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

        let mut cpu = Cpu::new(program);
        let insts_cnt = run_to_halt(&mut cpu);

        assert_eq!(insts_cnt, 24);
    }
}
