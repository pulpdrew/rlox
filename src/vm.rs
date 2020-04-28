use crate::error::ErrorHandler;
use crate::executable::Executable;
use crate::object::Obj;
use crate::token::Span;
use crate::value::Value;

use num_traits::FromPrimitive;
use std::collections::{HashMap, VecDeque};

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
}

#[derive(Debug)]
pub struct VM {
    ip: usize,
    chunk: Executable,
    stack: VecDeque<Value>,
    globals: HashMap<String, Value>,
}

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub span: Span,
}

impl VM {
    pub fn new() -> Self {
        VM {
            ip: 0,
            chunk: Executable::new(String::from("dummy chunk")),
            stack: VecDeque::new(),
            globals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, chunk: Executable, handler: &ErrorHandler) {
        self.ip = 0;
        self.chunk = chunk;

        match self.run() {
            Ok(()) => {}
            Err(e) => {
                handler.error(&e.span, e.message.as_str());
            }
        }
    }

    fn run(&mut self) -> Result<(), RuntimeError> {
        loop {
            if cfg!(disassemble) {
                self.chunk.disassemble_instruction(self.ip);
            }

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
                            span: self.chunk.spans[self.ip - 1],
                        });
                    }
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
                Some(OpCode::GetGlobal) => {
                    let index = self.read_byte() as u16;
                    let name_arg = self.chunk.get_constant(index).clone();
                    if let Value::Obj(Obj::String(name)) = name_arg {
                        let var_value = match self.globals.get(&*name) {
                            Some(value) => value.clone(),
                            None => {
                                return Err(RuntimeError {
                                    message: format!(
                                        "Attempted to get unknown variable {:?}",
                                        name
                                    ),
                                    span: self.chunk.spans[self.ip - 2],
                                })
                            }
                        };
                        self.push(var_value);
                    } else {
                        panic!("Attempt to assign to global {:?}", self.peek(0))
                    }
                }
                Some(OpCode::GetLongGlobal) => {
                    let index = (self.read_byte() * u8::max_value() + self.read_byte()) as u16;
                    let name_arg = self.chunk.get_constant(index).clone();
                    if let Value::Obj(Obj::String(name)) = name_arg {
                        let var_value = match self.globals.get(&*name) {
                            Some(value) => value.clone(),
                            None => {
                                return Err(RuntimeError {
                                    message: format!(
                                        "Attempted to get unknown variable {:?}",
                                        name
                                    ),
                                    span: self.chunk.spans[self.ip - 3],
                                })
                            }
                        };
                        self.push(var_value);
                    } else {
                        panic!("Attempt to assign to global {:?}", self.peek(0))
                    }
                }
                Some(OpCode::SetGlobal) => {
                    let index = self.read_byte() as u16;
                    if let Value::Obj(Obj::String(name)) = self.chunk.get_constant(index).clone() {
                        if self.globals.contains_key(&*name) {
                            self.globals
                                .insert(name.clone().to_string(), self.peek(0).clone());
                        } else {
                            return Err(RuntimeError {
                                message: format!("Assigned to undeclared global {:?}", name),
                                span: self.chunk.spans[self.ip - 2],
                            });
                        }
                    } else {
                        panic!("Invalid SetGlobal operand, references {:?}", self.peek(0))
                    }
                }
                Some(OpCode::SetLongGlobal) => {
                    let index = (self.read_byte() * u8::max_value() + self.read_byte()) as u16;
                    if let Value::Obj(Obj::String(name)) = self.chunk.get_constant(index).clone() {
                        if self.globals.contains_key(&*name) {
                            self.globals
                                .insert(name.clone().to_string(), self.peek(0).clone());
                        } else {
                            return Err(RuntimeError {
                                message: format!("Assigned to undeclared global {:?}", name),
                                span: self.chunk.spans[self.ip - 3],
                            });
                        }
                    } else {
                        panic!(
                            "Invalid SetLongGlobal operand, references {:?}",
                            self.peek(0)
                        )
                    }
                }
                Some(OpCode::DeclareGlobal) => {
                    let index = self.read_byte() as u16;
                    if let Value::Obj(Obj::String(name)) = self.chunk.get_constant(index).clone() {
                        self.globals.insert(name.clone().to_string(), Value::Nil);
                    } else {
                        panic!(
                            "Invalid SetLongGlobal operand, references {:?}",
                            self.peek(0)
                        )
                    }
                }
                Some(OpCode::DeclareLongGlobal) => {
                    let index = (self.read_byte() * u8::max_value() + self.read_byte()) as u16;
                    if let Value::Obj(Obj::String(name)) = self.chunk.get_constant(index).clone() {
                        self.globals.insert(name.clone().to_string(), Value::Nil);
                    } else {
                        panic!(
                            "Invalid SetLongGlobal operand, references {:?}",
                            self.peek(0)
                        )
                    }
                }
                None => {
                    return Err(RuntimeError {
                        message: format!(
                            "Unrecognized bytecode {} at offset {}",
                            self.chunk[self.ip], self.ip
                        ),
                        span: self.chunk.spans[self.ip],
                    })
                }
            }
            if cfg!(feature = "disassemble") {
                self.print_stack();
                println!(" Globals: {:?}", self.globals);
                println!();
            }
        }
    }

    fn binary_op(&mut self, op: &OpCode) -> Result<(), RuntimeError> {
        let right = self.peek(0).clone();
        let left = self.peek(1).clone();

        // Check for numeric operands, when apropriate
        match op {
            OpCode::Subtract
            | OpCode::Multiply
            | OpCode::Divide
            | OpCode::Less
            | OpCode::LessEqual
            | OpCode::Greater
            | OpCode::GreaterEqual => {
                if !left.is_number() || !right.is_number() {
                    return Err(RuntimeError {
                        message: format!("Cannot apply {:?} to non-numeric types", op),
                        span: self.chunk.spans[self.ip - 1],
                    });
                }
            }
            OpCode::Add => match left {
                Value::Number(_) => match right {
                    Value::Number(_) => {}
                    _ => {
                        return Err(RuntimeError {
                            message: String::from("Cannot apply '+' to Number and Non-Number"),
                            span: self.chunk.spans[self.ip - 1],
                        });
                    }
                },
                Value::Obj(Obj::String(_)) => match right {
                    Value::Obj(Obj::String(_)) => {}
                    _ => {
                        return Err(RuntimeError {
                            message: String::from("Cannot apply '+' to String and Non-String"),
                            span: self.chunk.spans[self.ip - 1],
                        });
                    }
                },
                _ => {
                    return Err(RuntimeError {
                        message: String::from("Cannot apply '+' to non-numeric or non-string type"),
                        span: self.chunk.spans[self.ip - 1],
                    });
                }
            },
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
        print!(" Stack: ");
        for v in &self.stack {
            print!("[{:?}] ", v)
        }
        println!();
    }
}
