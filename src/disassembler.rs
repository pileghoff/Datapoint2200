use crate::cpu::*;
#[derive(Debug, Clone)]
struct Disassembler {
    addr_to_line: Vec<(u16, String)>,
}

const FLAG_NAME: [&str; 4] = ["Cf", "Zf", "Sf", "Pf"];
const REG_NAME: [&str; 8] = ["A", "B", "C", "D", "E", "H", "L", "M"];

impl Disassembler {
    fn new(mut cpu: Cpu) -> Disassembler {
        let mut disassembler = Disassembler {
            addr_to_line: Vec::new(),
        };
        cpu.program_counter = 0;
        while cpu.program_counter as usize <= cpu.memory.len() {
            fetch_instruction(&mut cpu);
            let inst = cpu.instruction_register;
            let d = REG_NAME[inst.get_destination() as usize];
            let s = REG_NAME[inst.get_source() as usize];
            let c = FLAG_NAME[inst.get_destination() as usize];
            let op = inst.operand;
            let addr = inst.address;
            let line = (match inst.instruction_type {
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
                    format!("JumpIfNor {}, {}", c, addr.unwrap())
                }
                crate::instruction::InstructionType::Call => format!("Call {}", addr.unwrap()),
                crate::instruction::InstructionType::CallIf => {
                    format!("CallIf {}, {}", c, addr.unwrap())
                }
                crate::instruction::InstructionType::CallIfNot => {
                    format!("CallIfNot {}, {}", c, addr.unwrap())
                }
                crate::instruction::InstructionType::Return => format!("Return"),
                crate::instruction::InstructionType::ReturnIf => format!("ReturnIf {}", c),
                crate::instruction::InstructionType::ReturnIfNot => format!("ReturnIfNot {}", c),
                crate::instruction::InstructionType::ShiftRight => format!("ShiftRight"),
                crate::instruction::InstructionType::ShiftLeft => format!("ShiftLeft"),
                crate::instruction::InstructionType::Nop => format!("Nop"),
                crate::instruction::InstructionType::Halt => format!("Halt"),
                crate::instruction::InstructionType::Input => format!("Input"),
                crate::instruction::InstructionType::Pop => format!("Pop"),
                crate::instruction::InstructionType::Push => format!("Push",),
                crate::instruction::InstructionType::EnableIntr => format!("EnbleIntr",),
                crate::instruction::InstructionType::DisableInts => format!("DisableIntr",),
                crate::instruction::InstructionType::SelectAlpha => format!("SelecctAlpha",),
                crate::instruction::InstructionType::SelectBeta => format!("SelectBeta"),
                crate::instruction::InstructionType::Ex => format!("Ex"),
            });

            disassembler.addr_to_line.push((cpu.program_counter, line));
        }
        disassembler
    }
}
