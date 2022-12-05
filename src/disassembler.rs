use std::cmp::max;

use crate::{
    cpu::*,
    instruction::{Instruction, InstructionType, FLAG_NAME, REG_NAME},
};
#[derive(Debug, Clone)]
pub struct Disassembler {
    addr_to_line: Vec<(u16, String)>,
}

impl Disassembler {
    pub fn new(mut cpu: Cpu) -> Disassembler {
        let mut disassembler = Disassembler {
            addr_to_line: Vec::new(),
        };
        cpu.program_counter = 0;
        let mut inst = Instruction {
            instruction_type: InstructionType::Add,
            opcode: 0,
            operand: None,
            address: None,
        };
        while inst.instruction_type != InstructionType::Unknown {
            let program_counter = cpu.program_counter;
            inst = cpu.fetch_instruction();
            let d = REG_NAME[inst.get_destination() as usize];
            let s = REG_NAME[inst.get_source() as usize];
            let c = FLAG_NAME[inst.get_destination() as usize];
            let op = inst.operand;
            let addr = inst.address;
            let line = match inst.instruction_type {
                crate::instruction::InstructionType::Unknown => "Unknown".to_string(),
                crate::instruction::InstructionType::LoadImm => {
                    format!("LoadImm {}, {}", d, op.unwrap())
                }
                crate::instruction::InstructionType::Load => format!("Load {}", s),
                crate::instruction::InstructionType::AddImm => format!("AddImm {}", op.unwrap()),
                crate::instruction::InstructionType::Add => format!("Add {}", s),
                crate::instruction::InstructionType::AddImmCarry => {
                    format!("AddImmCarry {}", op.unwrap())
                }
                crate::instruction::InstructionType::AddCarry => format!("AddCarry {}", d),
                crate::instruction::InstructionType::SubImm => format!("SubImm {}", op.unwrap()),
                crate::instruction::InstructionType::Sub => format!("Sub {}", d),
                crate::instruction::InstructionType::SubImmBorrow => {
                    format!("SubImmBorror {}", op.unwrap())
                }
                crate::instruction::InstructionType::SubBorrow => format!("SubBorrow {}", d),
                crate::instruction::InstructionType::AndImm => format!("AndImm {}", op.unwrap()),
                crate::instruction::InstructionType::And => format!("And {}", d),
                crate::instruction::InstructionType::OrImm => format!("OrImm {}", op.unwrap()),
                crate::instruction::InstructionType::Or => format!("Or {}", d),
                crate::instruction::InstructionType::XorImm => format!("XorImm {}", op.unwrap()),
                crate::instruction::InstructionType::Xor => format!("Xor {}", d),
                crate::instruction::InstructionType::CompImm => format!("CompImm {}", op.unwrap()),
                crate::instruction::InstructionType::Comp => format!("Comp {}", d),
                crate::instruction::InstructionType::Jump => format!("Jump {}", addr.unwrap()),
                crate::instruction::InstructionType::JumpIf => {
                    format!("JumpIf {}, {}", c, addr.unwrap())
                }
                crate::instruction::InstructionType::JumpIfNot => {
                    format!("JumpIfNot {}, {}", c, addr.unwrap())
                }
                crate::instruction::InstructionType::Call => format!("Call {}", addr.unwrap()),
                crate::instruction::InstructionType::CallIf => {
                    format!("CallIf {}, {}", c, addr.unwrap())
                }
                crate::instruction::InstructionType::CallIfNot => {
                    format!("CallIfNot {}, {}", c, addr.unwrap())
                }
                crate::instruction::InstructionType::Return => "Return".to_string(),
                crate::instruction::InstructionType::ReturnIf => format!("ReturnIf {}", c),
                crate::instruction::InstructionType::ReturnIfNot => format!("ReturnIfNot {}", c),
                crate::instruction::InstructionType::ShiftRight => "ShiftRight".to_string(),
                crate::instruction::InstructionType::ShiftLeft => "ShiftLeft".to_string(),
                crate::instruction::InstructionType::Nop => "Nop".to_string(),
                crate::instruction::InstructionType::Halt => "Halt".to_string(),
                crate::instruction::InstructionType::Input => "Input".to_string(),
                crate::instruction::InstructionType::Pop => "Pop".to_string(),
                crate::instruction::InstructionType::Push => "Push".to_string(),
                crate::instruction::InstructionType::EnableIntr => "EnbleIntr".to_string(),
                crate::instruction::InstructionType::DisableInts => "DisableIntr".to_string(),
                crate::instruction::InstructionType::SelectAlpha => "SelecctAlpha".to_string(),
                crate::instruction::InstructionType::SelectBeta => "SelectBeta".to_string(),
                crate::instruction::InstructionType::Adr => todo!(),
                crate::instruction::InstructionType::Status => todo!(),
                crate::instruction::InstructionType::Data => todo!(),
                crate::instruction::InstructionType::Write => todo!(),
                crate::instruction::InstructionType::Com1 => todo!(),
                crate::instruction::InstructionType::Com2 => todo!(),
                crate::instruction::InstructionType::Com3 => todo!(),
                crate::instruction::InstructionType::Com4 => todo!(),
                crate::instruction::InstructionType::Beep => todo!(),
                crate::instruction::InstructionType::Click => todo!(),
                crate::instruction::InstructionType::Deck1 => todo!(),
                crate::instruction::InstructionType::Deck2 => todo!(),
                crate::instruction::InstructionType::Rbk => todo!(),
                crate::instruction::InstructionType::Wbk => todo!(),
                crate::instruction::InstructionType::Bsp => todo!(),
                crate::instruction::InstructionType::Sf => todo!(),
                crate::instruction::InstructionType::Sb => todo!(),
                crate::instruction::InstructionType::Rewind => todo!(),
                crate::instruction::InstructionType::Tstop => todo!(),
            };

            disassembler.addr_to_line.push((program_counter, line));
        }
        disassembler
    }

    fn find_index(&self, addr: usize) -> Option<usize> {
        for (i, (a, _)) in self.addr_to_line.iter().enumerate() {
            if addr == a.to_owned() as usize {
                return Some(i);
            }
        }
        None
    }

    pub fn get_lines(&self, addr: u16, num_lines: usize, num_lines_before: usize) -> Vec<String> {
        let mut res = Vec::new();
        let i = self.find_index(addr as usize).unwrap();
        let start_index = max(0, (i as i32) - (num_lines_before as i32)) as usize;
        for i in start_index..start_index + num_lines {
            if let Some((a, l)) = self.addr_to_line.get(i) {
                let mut c = ":";
                if addr == *a {
                    c = ">";
                }
                res.push(format!("{:#06x}{} {}", a, c, l));
            } else {
                break;
            }
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::datapoint::Datapoint;

    use super::*;
    #[test]
    fn test_basics() {
        let program = vec!["LoadImm A, 10", "AddImm 246", "Halt"];
        let machine = Datapoint::new(program, 1.0);
        let disassembler = Disassembler::new(machine.cpu);
        let output = disassembler.get_lines(0, 3, 0);
        assert_eq!(
            output,
            vec![
                "0x0000> LoadImm A, 10",
                "0x0002: AddImm 246",
                "0x0004: Halt"
            ]
        );
    }

    #[test]
    fn test_lines_before() {
        let program = vec!["LoadImm A, 10", "AddImm 246", "Halt"];
        let machine = Datapoint::new(program, 1.0);
        let disassembler = Disassembler::new(machine.cpu);
        let output = disassembler.get_lines(2, 3, 1);
        assert_eq!(
            output,
            vec![
                "0x0000: LoadImm A, 10",
                "0x0002> AddImm 246",
                "0x0004: Halt"
            ]
        );
    }

    #[test]
    fn test_lines_before_neg() {
        let program = vec!["LoadImm A, 10", "AddImm 246", "Halt"];
        let machine = Datapoint::new(program, 1.0);
        let disassembler = Disassembler::new(machine.cpu);
        let output = disassembler.get_lines(2, 3, 3);
        assert_eq!(
            output,
            vec![
                "0x0000: LoadImm A, 10",
                "0x0002> AddImm 246",
                "0x0004: Halt"
            ]
        );
    }
}
