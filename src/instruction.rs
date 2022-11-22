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
    Ex,
}

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub instruction_type: InstructionType,
    pub opcode: u8,
    pub operand: Option<u8>,
    pub address: Option<u16>,
}

impl Instruction {
    pub fn get_instruction_type(&self) -> u8 {
        (self.opcode & 0xc0) >> 6
    }

    pub fn get_destination(&self) -> u8 {
        (self.opcode & 0x38) >> 3
    }

    pub fn get_source(&self) -> u8 {
        self.opcode & 0x07
    }
}
