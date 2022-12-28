#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionType {
    Unknown,
    LoadImm,
    Load,
    AddImm,
    Add,
    AddImmCarry,
    AddCarry,
    SubImm,
    Sub,
    SubImmBorrow,
    SubBorrow,
    AndImm,
    And,
    OrImm,
    Or,
    XorImm,
    Xor,
    CompImm,
    Comp,
    Jump,
    JumpIf,
    JumpIfNot,
    Call,
    CallIf,
    CallIfNot,
    Return,
    ReturnIf,
    ReturnIfNot,
    ShiftRight,
    ShiftLeft,
    Nop,
    Halt,
    Input,
    Pop,
    Push,
    EnableIntr,
    DisableInts,
    SelectAlpha,
    SelectBeta,
    // Ex commands
    Adr,
    Status,
    Data,
    Write,
    Com1,
    Com2,
    Com3,
    Com4,
    Beep,
    Click,
    Deck1,
    Deck2,
    Rbk,
    Wbk,
    Bsp,
    Sf,
    Sb,
    Rewind,
    Tstop,
}

pub const FLAG_NAME: [&str; 8] = ["Cf", "Zf", "Sf", "Pf", "_", "_", "_", "_"];
pub const REG_NAME: [&str; 8] = ["A", "B", "C", "D", "E", "H", "L", "M"];

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub instruction_type: InstructionType,
    pub opcode: u8,
    pub operand: Option<u8>,
    pub address: Option<u16>,
}

impl Instruction {
    pub fn unknown() -> Instruction {
        Instruction {
            instruction_type: InstructionType::Unknown,
            opcode: 0,
            operand: None,
            address: None,
        }
    }
    pub fn get_instruction_type(&self) -> u8 {
        (self.opcode & 0xc0) >> 6
    }

    pub fn get_destination(&self) -> u8 {
        (self.opcode & 0x38) >> 3
    }

    pub fn get_source(&self) -> u8 {
        self.opcode & 0x07
    }

    pub fn get_clock_cycles(&self) -> usize {
        match self.instruction_type {
            InstructionType::Unknown => 0,
            InstructionType::LoadImm => 2,
            InstructionType::Load => 2,
            InstructionType::AddImm => 3,
            InstructionType::Add => 2,
            InstructionType::AddImmCarry => 3,
            InstructionType::AddCarry => 2,
            InstructionType::SubImm => 3,
            InstructionType::Sub => 2,
            InstructionType::SubImmBorrow => 3,
            InstructionType::SubBorrow => 2,
            InstructionType::AndImm => 3,
            InstructionType::And => 2,
            InstructionType::OrImm => 3,
            InstructionType::Or => 2,
            InstructionType::XorImm => 3,
            InstructionType::Xor => 2,
            InstructionType::CompImm => 3,
            InstructionType::Comp => 2,
            InstructionType::Jump => 4,
            InstructionType::JumpIf => 4,
            InstructionType::JumpIfNot => 4,
            InstructionType::Call => 4,
            InstructionType::CallIf => 4,
            InstructionType::CallIfNot => 4,
            InstructionType::Return => 2,
            InstructionType::ReturnIf => 2,
            InstructionType::ReturnIfNot => 2,
            InstructionType::ShiftRight => 2,
            InstructionType::ShiftLeft => 2,
            InstructionType::Nop => 2,
            InstructionType::Halt => 0,
            InstructionType::Input => 6,
            InstructionType::Pop => 3,
            InstructionType::Push => 2,
            InstructionType::EnableIntr => 2,
            InstructionType::DisableInts => 2,
            InstructionType::SelectAlpha => 2,
            InstructionType::SelectBeta => 2,
            _ => 6, // All ex instructions
        }
    }
}
