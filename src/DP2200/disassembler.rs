use crate::DP2200::{
    cpu::Cpu,
    datapoint::Datapoint,
    instruction::{Instruction, InstructionType, FLAG_NAME, REG_NAME},
};

pub fn disassemble(memory: &[u8]) -> Vec<(u16, String)> {
    let mut datapoint = Datapoint::build(memory, 1.0);
    let len = memory.len() as u16;
    let mut cpu = datapoint.cpu;
    let mut addr_to_line = Vec::new();

    cpu.program_counter = 0;
    let mut inst = Instruction {
        instruction_type: InstructionType::Add,
        opcode: 0,
        operand: None,
        address: None,
    };
    while inst.instruction_type != InstructionType::Unknown && cpu.program_counter < len {
        let program_counter = cpu.program_counter;
        let tmp_inst = cpu.fetch_instruction();
        if tmp_inst.is_none() {
            break;
        }
        inst = tmp_inst.unwrap();
        let d = REG_NAME[inst.get_destination() as usize];
        let s = REG_NAME[inst.get_source() as usize];

        let c = inst.get_destination();
        let c = if c >= 4 { c - 4 } else { c };
        let c = FLAG_NAME[c as usize];
        let op = inst.operand;
        let addr = inst.address;
        let line = match inst.instruction_type {
            InstructionType::Unknown => format!("{:#02x}", inst.opcode),
            InstructionType::LoadImm => format!("LoadImm {}, {}", d, op.unwrap()),
            InstructionType::Load => format!("Load {}, {}", d, s),
            InstructionType::AddImm => format!("AddImm {}", op.unwrap()),
            InstructionType::Add => format!("Add {}", s),
            InstructionType::AddImmCarry => {
                format!("AddImmCarry {}", op.unwrap())
            }
            InstructionType::AddCarry => format!("AddCarry {}", s),
            InstructionType::SubImm => format!("SubImm {}", op.unwrap()),
            InstructionType::Sub => format!("Sub {}", s),
            InstructionType::SubImmBorrow => {
                format!("SubImmBorror {}", op.unwrap())
            }
            InstructionType::SubBorrow => format!("SubBorrow {}", s),
            InstructionType::AndImm => format!("AndImm {}", op.unwrap()),
            InstructionType::And => format!("And {}", s),
            InstructionType::OrImm => format!("OrImm {}", op.unwrap()),
            InstructionType::Or => format!("Or {}", s),
            InstructionType::XorImm => format!("XorImm {}", op.unwrap()),
            InstructionType::Xor => format!("Xor {}", s),
            InstructionType::CompImm => format!("CompImm {}", op.unwrap()),
            InstructionType::Comp => format!("Comp {}", s),
            InstructionType::Jump => format!("Jump {:#04x}", addr.unwrap()),
            InstructionType::JumpIf => {
                format!("JumpIf {}, {:#04x}", c, addr.unwrap())
            }
            InstructionType::JumpIfNot => {
                format!("JumpIfNot {}, {:#04x}", c, addr.unwrap())
            }
            InstructionType::Call => format!("Call {:#04x}", addr.unwrap()),
            InstructionType::CallIf => {
                format!("CallIf {}, {:#04x}", c, addr.unwrap())
            }
            InstructionType::CallIfNot => {
                format!("CallIfNot {}, {:#04x}", c, addr.unwrap())
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

        addr_to_line.push((program_counter, line));
    }
    addr_to_line
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
        let output = disassemble(&machine.cpu.memory);
        assert_eq!(
            output[0..3],
            vec![
                (0x0000, "LoadImm A, 10".to_string()),
                (0x0002, "AddImm 246".to_string()),
                (0x0004, "Halt".to_string())
            ]
        );
    }
}
