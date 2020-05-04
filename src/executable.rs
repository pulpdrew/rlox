use crate::opcode::OpCode;
use crate::token::Span;
use crate::value::Value;

/// An Executable contains the output of compilation to be run on a VM.
#[derive(Debug)]
pub struct Executable {
    /// The OpCodes and arguments to be executed
    code: Vec<u8>,

    /// The static Values referenced by the executable code
    constants: Vec<Value>,

    /// The source line numbers associated with each OpCode.
    /// `lines[i]` is the source line number of `code[i]`.
    pub spans: Vec<Span>,

    /// The name of the executable unit. Could be a function name or <script>
    pub name: String,
}

#[allow(clippy::len_without_is_empty)]
impl Executable {
    /// Create a new, empty Executable with the given name
    pub fn new(name: String) -> Self {
        Executable {
            code: vec![],
            spans: vec![],
            constants: vec![],
            name,
        }
    }

    /// Append an OpCode to the Executable
    pub fn push_opcode(&mut self, code: OpCode, span: Span) {
        self.code.push(code.into());
        self.spans.push(span);
    }

    /// Append a u8 to the Executable, associated with the given span
    pub fn push_u8(&mut self, value: u8, span: Span) {
        self.code.push(value);
        self.spans.push(span);
    }

    /// Append a u16 to the Executable as two bytes, each associated with the given span
    pub fn push_u16(&mut self, value: u16, span: Span) {
        let high = (value >> 8) as u8;
        let low = value as u8;

        self.code.push(high);
        self.code.push(low);
        self.spans.push(span);
        self.spans.push(span);
    }

    /// Read a u8 from the executable at the given index
    pub fn read_u8(&self, index: usize) -> u8 {
        self.code[index]
    }

    /// Read a u16 from the executable (two u8s) at the given index
    pub fn read_u16(&self, index: usize) -> u16 {
        ((self.code[index] as u16) << 8) + self.code[index + 1] as u16
    }

    /// Replace a u8 in the executable with the given value
    pub fn replace_u8(&mut self, index: usize, value: u8) {
        self.code[index] = value;
    }

    /// Replace a u16 in the executable with the given value
    pub fn replace_u16(&mut self, index: usize, value: u16) {
        let high = (value >> 8) as u8;
        let low = value as u8;
        self.code[index] = high;
        self.code[index + 1] = low;
    }

    /// Add a constant to the list of constants and an instruction to
    /// access that constant.
    ///
    /// The executable may have no more that `u16::max_value()` constants.
    pub fn push_constant_inst(&mut self, op: OpCode, value: Value, span: Span) -> u16 {
        self.constants.push(value);

        let index: usize = self.constants.len() - 1;
        if index <= (u8::max_value() as usize) {
            self.push_u8(op.into(), span);
            self.push_u8(index as u8, span);
        } else if index <= u16::max_value() as usize {
            self.push_u8(op.into(), span);
            self.push_u16(index as u16, span);
        } else {
            eprintln!("Cannot have more than {} constants", u16::max_value())
        }

        index as u16
    }

    /// Retrieve a constant by index from the Executable's constants table.
    pub fn get_constant(&self, index: u16) -> &Value {
        &self.constants[index as usize]
    }

    /// The number of bytes (OpCodes + arguments) in the Executable
    pub fn len(&self) -> usize {
        self.code.len()
    }

    /// Disassemble this Executable and print the result
    pub fn dump(&self) {
        println!();
        println!("(Dumping: {})", self.name);
        println!("Index  OpCode              Arguments");
        println!("------------------------------------");
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
        println!();
    }

    /// Disassemble the Instruction beginning at the given offset
    /// and print the result
    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:0>5}  ", offset);
        match OpCode::from(self.read_u8(offset)) {
            OpCode::Constant => self.constant_instruction("Constant", offset),
            OpCode::LongConstant => self.long_constant_instruction("LongConstant", offset),
            OpCode::Return => self.simple_instruction("Return", offset),
            OpCode::Add => self.simple_instruction("Add", offset),
            OpCode::Subtract => self.simple_instruction("Subtract", offset),
            OpCode::Multiply => self.simple_instruction("Multiply", offset),
            OpCode::Divide => self.simple_instruction("Divide", offset),
            OpCode::Negate => self.simple_instruction("Negate", offset),
            OpCode::Less => self.simple_instruction("Less", offset),
            OpCode::LessEqual => self.simple_instruction("LessEqual", offset),
            OpCode::Greater => self.simple_instruction("Greater", offset),
            OpCode::GreaterEqual => self.simple_instruction("GreaterEqual", offset),
            OpCode::Equal => self.simple_instruction("Equal", offset),
            OpCode::Not => self.simple_instruction("Not", offset),
            OpCode::Print => self.simple_instruction("Print", offset),
            OpCode::Pop => self.simple_instruction("Pop", offset),
            OpCode::SetGlobal => self.constant_instruction("SetGlobal", offset),
            OpCode::SetLongGlobal => self.long_constant_instruction("SetLongGlobal", offset),
            OpCode::GetGlobal => self.constant_instruction("GetGlobal", offset),
            OpCode::GetLongGlobal => self.long_constant_instruction("GetLongGlobal", offset),
            OpCode::DeclareGlobal => self.constant_instruction("DeclareGlobal", offset),
            OpCode::DeclareLongGlobal => {
                self.long_constant_instruction("DeclareLongGlobal", offset)
            }
            OpCode::SetLocal => self.single_arg_instruction("SetLocal", offset),
            OpCode::GetLocal => self.single_arg_instruction("GetLocal", offset),
            OpCode::Jump => self.single_long_arg_instruction("Jump", offset),
            OpCode::JumpIfTrue => self.single_long_arg_instruction("JumpIfTrue", offset),
            OpCode::JumpIfFalse => self.single_long_arg_instruction("JumpIfFalse", offset),
            OpCode::Call => self.single_arg_instruction("Call", offset),
        }
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let index = self.read_u8(offset + 1);
        let value = &self.constants[index as usize];
        println!("{:<16} {:>4}[{:?}]", name, index, value);
        offset + 2
    }
    fn single_arg_instruction(&self, name: &str, offset: usize) -> usize {
        let arg = self.read_u8(offset + 1);
        println!("{:<16} {:>4}", name, arg);
        offset + 2
    }
    fn single_long_arg_instruction(&self, name: &str, offset: usize) -> usize {
        let arg = self.read_u16(offset + 1);
        println!("{:<16} {:>4}", name, arg);
        offset + 3
    }
    fn long_constant_instruction(&self, name: &str, offset: usize) -> usize {
        let index = self.read_u16(offset + 1);
        let value = &self.constants[index as usize];
        println!("{:<16} {:>4}[{:?}]", name, index, value);
        offset + 3
    }
    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{0:<16}", name);
        offset + 1
    }
}
