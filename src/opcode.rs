use num_derive::FromPrimitive;
use num_derive::ToPrimitive;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;

#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq)]
pub enum OpCode {
    Constant,
    LongConstant,
    Return,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Not,
    Equal,
    Print,
    Pop,
    DeclareGlobal,
    DeclareLongGlobal,
    GetGlobal,
    SetGlobal,
    GetLongGlobal,
    SetLongGlobal,
    GetLocal,
    SetLocal,
    Jump,
    JumpIfTrue,
    JumpIfFalse,
    Call,
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        FromPrimitive::from_u8(byte)
            .unwrap_or_else(|| panic!("failed to convert {} into OpCode", byte))
    }
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        ToPrimitive::to_u8(&self).unwrap_or_else(|| panic!("failed to convert {:?} into u8", self))
    }
}
