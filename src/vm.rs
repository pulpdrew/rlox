use crate::chunk::Chunk;
use crate::opcode::OpCode;
use crate::value::Value;

use num_traits::FromPrimitive;
use std::collections::VecDeque;

pub struct VM {
    ip: usize,
    chunk: Chunk,
    stack: VecDeque<Value>,
}

pub struct RuntimeError {
    pub message: String,
    pub line: i32,
}

impl VM {
    pub fn new() -> Self {
        VM {
            ip: 0,
            chunk: Chunk::new(String::from("dummy chunk")),
            stack: VecDeque::new(),
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> Result<(), RuntimeError> {
        self.chunk = chunk;
        self.run()
    }

    fn run<'a>(&mut self) -> Result<(), RuntimeError> {
        loop {
            self.chunk.disassemble_instruction(self.ip);
            match FromPrimitive::from_u8(self.read_byte()) {
                Some(OpCode::Constant) => {
                    let index = self.read_byte() as u16;
                    self.push(self.chunk.get_constant(index).clone());
                }
                Some(OpCode::LongConstant) => {
                    let index = (self.read_byte() * u8::max_value() + self.read_byte()) as u16;
                    self.push(self.chunk.get_constant(index).clone());
                }
                Some(OpCode::Add) => {
                    let right = self.pop();
                    let left = self.pop();

                    if left.is_number() && right.is_number() {
                        self.push(left + right);
                    } else {
                        return Err(RuntimeError {
                            message: String::from("Cannot add non-numeric types"),
                            line: self.chunk.lines[self.ip - 1],
                        });
                    }
                }
                Some(OpCode::Subtract) => {
                    let right = self.pop();
                    let left = self.pop();

                    if left.is_number() && right.is_number() {
                        self.push(left - right);
                    } else {
                        return Err(RuntimeError {
                            message: String::from("Cannot subtract non-numeric types"),
                            line: self.chunk.lines[self.ip - 1],
                        });
                    }
                }
                Some(OpCode::Multiply) => {
                    let right = self.pop();
                    let left = self.pop();

                    if left.is_number() && right.is_number() {
                        self.push(left * right);
                    } else {
                        return Err(RuntimeError {
                            message: String::from("Cannot multiply non-numeric types"),
                            line: self.chunk.lines[self.ip - 1],
                        });
                    }
                }
                Some(OpCode::Divide) => {
                    let right = self.pop();
                    let left = self.pop();

                    if left.is_number() && right.is_number() {
                        self.push(left / right);
                    } else {
                        return Err(RuntimeError {
                            message: String::from("Cannot divide non-numeric types"),
                            line: self.chunk.lines[self.ip - 1],
                        });
                    }
                }
                Some(OpCode::Negate) => {}
                Some(OpCode::Return) => return Ok(()),
                None => {
                    return Err(RuntimeError {
                        message: String::from(format!(
                            "Unrecognized bytecode {} at offset {}",
                            self.chunk[self.ip], self.ip
                        )),
                        line: self.chunk.lines[self.ip],
                    })
                }
            }
            self.print_stack();
        }
    }

    fn read_byte(&mut self) -> u8 {
        if self.ip >= self.chunk.len() {
            panic!(
                "read_byte out of bounds. chunk: {}, ip: {}",
                self.chunk.name, self.ip
            );
        }
        self.ip += 1;
        self.chunk[self.ip - 1]
    }

    fn push(&mut self, value: Value) {
        self.stack.push_back(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop_back().expect("Popped an emtpy stack")
    }

    fn print_stack(&self) {
        print!("Stack: ");
        for v in &self.stack {
            print!("[{:?}] ", v)
        }
        println!();
        println!();
    }
}
