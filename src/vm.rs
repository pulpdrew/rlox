use crate::executable::Executable;
use crate::value::Value;

use num_traits::FromPrimitive;
use std::collections::VecDeque;

#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum OpCode {
    Constant,
    True,
    False,
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
}

#[derive(Debug)]
pub struct VM {
    ip: usize,
    chunk: Executable,
    stack: VecDeque<Value>,
}

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub line: usize,
}

impl VM {
    pub fn new() -> Self {
        VM {
            ip: 0,
            chunk: Executable::new(String::from("dummy chunk")),
            stack: VecDeque::new(),
        }
    }

    pub fn interpret(&mut self, chunk: Executable) -> Result<(), RuntimeError> {
        self.chunk = chunk;
        self.run()
    }

    fn run<'a>(&mut self) -> Result<(), RuntimeError> {
        loop {
            self.chunk.disassemble_instruction(self.ip);

            let op = FromPrimitive::from_u8(self.read_byte());
            match op {
                Some(OpCode::Constant) => {
                    let index = self.read_byte() as u16;
                    self.push(self.chunk.get_constant(index).clone());
                }
                Some(OpCode::LongConstant) => {
                    let index = (self.read_byte() * u8::max_value() + self.read_byte()) as u16;
                    self.push(self.chunk.get_constant(index).clone());
                }
                Some(OpCode::Negate) => {
                    if self.peek(0).is_number() {
                        let value = -self.peek(0).clone();
                        self.pop();
                        self.push(value);
                    } else {
                        return Err(RuntimeError {
                            message: String::from("Cannot negate non-numeric types"),
                            line: self.chunk.lines[self.ip - 1],
                        });
                    }
                }
                Some(OpCode::True) => {
                    self.push(Value::Bool(true));
                }
                Some(OpCode::False) => {
                    self.push(Value::Bool(false));
                }
                Some(OpCode::Pop) => {
                    self.pop();
                }
                Some(OpCode::Not) => {
                    let right = self.peek(0);
                    let value = Value::Bool(!right.is_truthy());
                    self.pop();
                    self.push(value);
                }
                Some(OpCode::Return) => return Ok(()),
                Some(OpCode::Add)
                | Some(OpCode::Subtract)
                | Some(OpCode::Multiply)
                | Some(OpCode::Divide)
                | Some(OpCode::Less)
                | Some(OpCode::LessEqual)
                | Some(OpCode::Greater)
                | Some(OpCode::GreaterEqual)
                | Some(OpCode::Equal) => match self.binary_op(&op.unwrap()) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                },
                Some(OpCode::Print) => println!("{:}", self.pop()),
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

    fn binary_op(&mut self, op: &OpCode) -> Result<(), RuntimeError> {
        let right = self.peek(0).clone();
        let left = self.peek(1).clone();

        // Check for numeric operands, when apropriate
        match op {
            OpCode::Add
            | OpCode::Subtract
            | OpCode::Multiply
            | OpCode::Divide
            | OpCode::Less
            | OpCode::LessEqual
            | OpCode::Greater
            | OpCode::GreaterEqual => {
                if !left.is_number() || !right.is_number() {
                    return Err(RuntimeError {
                        message: format!("Cannot apply {:?} non-numeric types", op),
                        line: self.chunk.lines[self.ip - 1],
                    });
                }
            }
            _ => {}
        }

        let value = match op {
            OpCode::Add => left + right,
            OpCode::Subtract => left - right,
            OpCode::Multiply => left * right,
            OpCode::Divide => left / right,
            OpCode::Less => Value::Bool(left < right),
            OpCode::LessEqual => Value::Bool(left <= right),
            OpCode::Greater => Value::Bool(left > right),
            OpCode::GreaterEqual => Value::Bool(left >= right),
            OpCode::Equal => Value::Bool(left == right),
            _ => panic!("Invalid binary operation {:?}", op),
        };
        self.pop();
        self.pop();
        self.push(value);
        Ok(())
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
        self.stack.pop_back().expect("Popped an empty stack")
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack.len() - distance - 1]
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
