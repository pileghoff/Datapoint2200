use std::cmp::max;

use crate::DP2200::{
    cpu::*,
    instruction::{Instruction, InstructionType, FLAG_NAME, REG_NAME},
};

use super::datapoint::Datapoint;
#[derive(Debug, Clone)]
pub struct Disassembler {
    pub addr_to_line: Vec<(u16, String)>,
}

impl Disassembler {
    pub fn from_vec(memory: &[u8]) -> Disassembler {
        let datapoint = Datapoint::build(memory, 1.0);
        Disassembler::new(datapoint.cpu)
    }
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

            let c = inst.get_destination();
            let c = if c >= 4 { c - 4 } else { c };
            let c = FLAG_NAME[c as usize];
            let op = inst.operand;
            let addr = inst.address;
            let line = match inst.instruction_type {
                InstructionType::Unknown => format!("{:#02x}", inst.opcode),
                InstructionType::LoadImm => {
                    format!("LoadImm {}, {}", d, op.unwrap())
                }
                InstructionType::Load => format!("Load {}, {}", d, s),
                InstructionType::AddImm => format!("AddImm {}", op.unwrap()),
                InstructionType::Add => format!("Add {}", s),
                InstructionType::AddImmCarry => {
                    format!("AddImmCarry {}", op.unwrap())
                }
                InstructionType::AddCarry => format!("AddCarry {}", d),
                InstructionType::SubImm => format!("SubImm {}", op.unwrap()),
                InstructionType::Sub => format!("Sub {}", d),
                InstructionType::SubImmBorrow => {
                    format!("SubImmBorror {}", op.unwrap())
                }
                InstructionType::SubBorrow => format!("SubBorrow {}", d),
                InstructionType::AndImm => format!("AndImm {}", op.unwrap()),
                InstructionType::And => format!("And {}", d),
                InstructionType::OrImm => format!("OrImm {}", op.unwrap()),
                InstructionType::Or => format!("Or {}", d),
                InstructionType::XorImm => format!("XorImm {}", op.unwrap()),
                InstructionType::Xor => format!("Xor {}", d),
                InstructionType::CompImm => format!("CompImm {}", op.unwrap()),
                InstructionType::Comp => format!("Comp {}", s),
                InstructionType::Jump => format!("Jump {}", addr.unwrap()),
                InstructionType::JumpIf => {
                    format!("JumpIf {}, {}", c, addr.unwrap())
                }
                InstructionType::JumpIfNot => {
                    format!("JumpIfNot {}, {}", c, addr.unwrap())
                }
                InstructionType::Call => format!("Call {}", addr.unwrap()),
                InstructionType::CallIf => {
                    format!("CallIf {}, {}", c, addr.unwrap())
                }
                InstructionType::CallIfNot => {
                    format!("CallIfNot {}, {}", c, addr.unwrap())
                }
                InstructionType::Return => "Return".to_string(),
                InstructionType::ReturnIf => format!("ReturnIf {}", c),
                InstructionType::ReturnIfNot => format!("ReturnIfNot {}", c),
                InstructionType::ShiftRight => "ShiftRight".to_string(),
                InstructionType::ShiftLeft => "ShiftLeft".to_string(),
                InstructionType::Nop => "Nop".to_string(),
                InstructionType::Halt => "Halt".to_string(),
                InstructionType::Input => "Input".to_string(),
                InstructionType::Pop => "Pop".to_string(),
                InstructionType::Push => "Push".to_string(),
                InstructionType::EnableIntr => "EnbleIntr".to_string(),
                InstructionType::DisableInts => "DisableIntr".to_string(),
                InstructionType::SelectAlpha => "SelecctAlpha".to_string(),
                InstructionType::SelectBeta => "SelectBeta".to_string(),
                InstructionType::Adr => "Adr".to_string(),
                InstructionType::Status => "Status".to_string(),
                InstructionType::Data => "Data".to_string(),
                InstructionType::Write => "Write".to_string(),
                InstructionType::Com1 => "Com1".to_string(),
                InstructionType::Com2 => "Com2".to_string(),
                InstructionType::Com3 => "Com3".to_string(),
                InstructionType::Com4 => "Com4".to_string(),
                InstructionType::Beep => "Beep".to_string(),
                InstructionType::Click => "Click".to_string(),
                InstructionType::Deck1 => "Deck1".to_string(),
                InstructionType::Deck2 => "Deck2".to_string(),
                InstructionType::Rbk => "Rbk".to_string(),
                InstructionType::Wbk => "Wbk".to_string(),
                InstructionType::Bsp => "Bsp".to_string(),
                InstructionType::Sf => "Sf".to_string(),
                InstructionType::Sb => "Sb".to_string(),
                InstructionType::Rewind => "Rewing".to_string(),
                InstructionType::Tstop => "Tstop".to_string(),
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

    use crate::DP2200::datapoint::Datapoint;

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
