use crate::opcode::OpCode;
use crate::token::Span;
use crate::value::Value;
use std::io::Write;
use std::ops::Index;
use std::ops::IndexMut;

/// An Executable contains the output of compilation to be run on a VM.
#[derive(Debug, PartialEq)]
pub struct Executable {
    /// The OpCodes and arguments to be executed
    code: Vec<OpCode>,

    /// The static Values referenced by the executable code
    constants: Vec<Value>,

    /// The source line numbers associated with each OpCode.
    /// `lines[i]` is the source line number of `code[i]`.
    pub spans: Vec<Span>,

    /// The name of the executable unit. Could be a function name or <script>
    pub name: String,
}

impl Index<usize> for Executable {
    type Output = OpCode;
    fn index(&self, index: usize) -> &Self::Output {
        &self.code[index]
    }
}

impl IndexMut<usize> for Executable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.code[index]
    }
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

    /// Append an OpCode to the Executable, returning its index
    pub fn push_opcode(&mut self, code: OpCode, span: Span) -> usize {
        self.code.push(code);
        self.spans.push(span);
        self.code.len() - 1
    }

    /// Retrieve a constant by index from the Executable's constants table.
    pub fn get_constant(&self, index: usize) -> &Value {
        &self.constants[index]
    }

    /// Add a constant and return its index
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    /// The number of bytes (OpCodes + arguments) in the Executable
    pub fn len(&self) -> usize {
        self.code.len()
    }

    /// Disassemble this Executable and print the result
    pub fn dump<W: Write>(&self, out: &mut W) {
        writeln!(out).unwrap();
        writeln!(out, "(Dumping: {})", self.name).unwrap();
        writeln!(out, "Index  OpCode              Arguments").unwrap();
        writeln!(out, "------------------------------------").unwrap();
        for offset in 0..self.code.len() {
            self.disassemble_instruction(offset, out);
        }
        writeln!(out).unwrap();
    }

    pub fn disassemble_instruction<W: Write>(&self, offset: usize, out: &mut W) {
        write!(out, "{:0>5}  ", offset).unwrap();
        match self.code[offset] {
            OpCode::Constant(arg) => self.constant_instruction("Constant", arg, out),
            OpCode::Return => self.simple_instruction("Return", out),
            OpCode::Add => self.simple_instruction("Add", out),
            OpCode::Subtract => self.simple_instruction("Subtract", out),
            OpCode::Multiply => self.simple_instruction("Multiply", out),
            OpCode::Divide => self.simple_instruction("Divide", out),
            OpCode::Negate => self.simple_instruction("Negate", out),
            OpCode::Less => self.simple_instruction("Less", out),
            OpCode::Greater => self.simple_instruction("Greater", out),
            OpCode::LessEqual => self.simple_instruction("LessEqual", out),
            OpCode::GreaterEqual => self.simple_instruction("GreaterEqual", out),
            OpCode::Not => self.simple_instruction("Not", out),
            OpCode::Equal => self.simple_instruction("Equal", out),
            OpCode::NotEqual => self.simple_instruction("NotEqual", out),
            OpCode::Print => self.simple_instruction("Print", out),
            OpCode::Pop => self.simple_instruction("Pop", out),
            OpCode::DeclareGlobal(arg) => self.constant_instruction("DeclareGlobal", arg, out),
            OpCode::GetGlobal(arg) => self.constant_instruction("GetGlobal", arg, out),
            OpCode::SetGlobal(arg) => self.constant_instruction("SetGlobal", arg, out),
            OpCode::GetLocal(arg) => self.single_arg_instruction("GetLocal", arg, out),
            OpCode::SetLocal(arg) => self.single_arg_instruction("SetLocal", arg, out),
            OpCode::GetSuper(arg) => self.constant_instruction("GetSuper", arg, out),
            OpCode::Jump(arg) => self.single_arg_instruction("Jump", arg, out),
            OpCode::JumpIfTrue(arg) => self.single_arg_instruction("JumpIfTrue", arg, out),
            OpCode::JumpIfFalse(arg) => self.single_arg_instruction("JumpIfFalse", arg, out),
            OpCode::Invoke(arg) => self.single_arg_instruction("Invoke", arg, out),
            OpCode::Closure(arg) => self.constant_instruction("Closure", arg, out),
            OpCode::GetUpvalue(arg) => self.single_arg_instruction("GetUpvalue", arg, out),
            OpCode::SetUpvalue(arg) => self.single_arg_instruction("SetUpvalue", arg, out),
            OpCode::ReadField(arg) => self.constant_instruction("ReadField", arg, out),
            OpCode::SetField(arg) => self.constant_instruction("SetField", arg, out),
            OpCode::Method => self.simple_instruction("Method", out),
            OpCode::Inherit => self.simple_instruction("Inherit", out),
            OpCode::Bool => self.simple_instruction("Bool", out),
        }
    }

    fn simple_instruction<W: Write>(&self, name: &str, out: &mut W) {
        writeln!(out, "{0:<16}", name).unwrap();
    }
    fn constant_instruction<W: Write>(&self, name: &str, index: usize, out: &mut W) {
        let value = &self.constants[index as usize];
        writeln!(out, "{:<16} {:>4}[{:?}]", name, index, value).unwrap();
    }
    fn single_arg_instruction<W: Write>(&self, name: &str, arg: usize, out: &mut W) {
        writeln!(out, "{:<16} {:>4}", name, arg).unwrap();
    }
}
