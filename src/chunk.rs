use crate::value::Value;
use crate::vm::OpCode;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;
use std::ops::Index;

#[derive(Debug)]
pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Value>,
    pub lines: Vec<usize>,
    pub name: String,
}

impl Chunk {
    pub fn new(name: String) -> Self {
        Chunk {
            code: vec![],
            lines: vec![],
            constants: vec![],
            name,
        }
    }

    pub fn push_opcode(&mut self, code: OpCode, line: usize) {
        self.code.push(to_byte(code));
        self.lines.push(line);
    }

    pub fn push_constant(&mut self, value: Value, line: usize) -> u16 {
        self.constants.push(value);
        self.lines.push(line);

        let index: usize = self.constants.len() - 1;
        if index <= (u8::max_value() as usize) {
            self.code.push(to_byte(OpCode::Constant));
            self.code.push(index as u8);
            self.lines.push(line);
        } else if index <= u16::max_value() as usize {
            self.code.push(to_byte(OpCode::LongConstant));
            self.code.push((index / 256) as u8);
            self.code.push((index % 256) as u8);
            self.lines.push(line);
            self.lines.push(line);
        } else {
            eprintln!("Cannot have more than {} constants", u16::max_value())
        }

        index as u16
    }

    pub fn get_constant(&self, index: u16) -> &Value {
        &self.constants[index as usize]
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn dump(&self) {
        println!();
        println!("(Dumping: {})", self.name);
        println!("Index  Line   OpCode             Arguments");
        println!("-------------------------------------------");
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
        println!();
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:0>5}  ", offset);
        if offset == 0 || self.lines[offset] != self.lines[offset - 1] {
            print!("{:0>5}  ", self.lines[offset]);
        } else {
            print!("    |  ");
        }
        match FromPrimitive::from_u8(self[offset]) {
            Some(OpCode::Constant) => self.constant_instruction("Constant", offset),
            Some(OpCode::LongConstant) => self.long_constant_instruction("LongConstant", offset),
            Some(OpCode::Return) => self.simple_instruction("Return", offset),
            Some(OpCode::Add) => self.simple_instruction("Add", offset),
            Some(OpCode::Subtract) => self.simple_instruction("Subtract", offset),
            Some(OpCode::Multiply) => self.simple_instruction("Multiply", offset),
            Some(OpCode::Divide) => self.simple_instruction("Divide", offset),
            Some(OpCode::Negate) => self.simple_instruction("Negate", offset),
            Some(OpCode::Pop) => self.simple_instruction("Pop", offset),
            None => {
                println!("Unknown opcode {}", self[offset]);
                offset + 1
            }
        }
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let index = self[offset + 1];
        let value = &self.constants[index as usize];
        println!("{:<16} {:>4}[{:?}]", name, index, value);
        offset + 2
    }
    fn long_constant_instruction(&self, name: &str, offset: usize) -> usize {
        let index = self[offset + 1] * u8::max_value() + self[offset + 2];
        let value = &self.constants[index as usize];
        println!("{:<16} {:>4}[{:?}]", name, index, value);
        offset + 3
    }
    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{0:<16}", name);
        offset + 1
    }
}

impl Index<usize> for Chunk {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.code[index]
    }
}

fn to_byte(opcode: OpCode) -> u8 {
    ToPrimitive::to_u8(&opcode).expect("Could not convert OpCode to u8")
}
