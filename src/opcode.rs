#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum OpCode {
    Constant,
    LongConstant,
    Return,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
}
