use crate::executable::Executable;
use crate::object::ObjKind;
use crate::token::Span;
use crate::value::Value;

use num_derive::FromPrimitive;
use num_derive::ToPrimitive;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;
use std::collections::{HashMap, VecDeque};
use std::io::Write;

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

#[derive(Debug)]
pub struct VM {
    ip: usize,
    bin: Executable,
    stack: VecDeque<Value>,
    globals: HashMap<String, Value>,
}

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub span: Span,
}

impl Default for VM {
    fn default() -> Self {
        VM::new()
    }
}

impl VM {
    pub fn new() -> Self {
        VM {
            ip: 0,
            bin: Executable::new(String::new()),
            stack: VecDeque::new(),
            globals: HashMap::new(),
        }
    }

    pub fn interpret<W: Write>(
        &mut self,
        bin: Executable,
        output_stream: &mut W,
    ) -> Result<(), RuntimeError> {
        self.ip = 0;
        self.bin = bin;

        loop {
            if cfg!(feature = "disassemble") {
                self.bin.disassemble_instruction(self.ip);
            }

            let op = FromPrimitive::from_u8(self.read_byte());
            match op {
                Some(OpCode::Constant) => {
                    let index = self.read_byte() as u16;
                    self.push(self.bin.get_constant(index).clone());
                }
                Some(OpCode::LongConstant) => {
                    let index = (self.read_byte() * u8::max_value() + self.read_byte()) as u16;
                    self.push(self.bin.get_constant(index).clone());
                }
                Some(OpCode::Negate) => {
                    if self.peek(0).is_number() {
                        let value = -self.peek(0).clone();
                        self.pop();
                        self.push(value);
                    } else {
                        return Err(RuntimeError {
                            message: String::from("Cannot negate non-numeric types"),
                            span: self.bin.spans[self.ip - 1],
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
                Some(OpCode::Print) => {
                    writeln!(output_stream, "{:}", self.pop()).unwrap();
                    output_stream.flush().unwrap();
                }
                Some(OpCode::GetGlobal) => {
                    let index = self.read_byte() as u16;
                    let name_arg = self.bin.get_constant(index).clone();
                    if let Value::Obj(name, ObjKind::String) = name_arg {
                        let var_value = match self.globals.get(&*name.as_string().unwrap()) {
                            Some(value) => value.clone(),
                            None => {
                                return Err(RuntimeError {
                                    message: format!(
                                        "Attempted to get unknown variable {:?}",
                                        name
                                    ),
                                    span: self.bin.spans[self.ip - 2],
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
                    let name_arg = self.bin.get_constant(index).clone();
                    if let Value::Obj(name, ObjKind::String) = name_arg {
                        let var_value = match self.globals.get(&*name.as_string().unwrap()) {
                            Some(value) => value.clone(),
                            None => {
                                return Err(RuntimeError {
                                    message: format!(
                                        "Attempted to get unknown variable {:?}",
                                        name
                                    ),
                                    span: self.bin.spans[self.ip - 3],
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
                    if let Value::Obj(name, ObjKind::String) = self.bin.get_constant(index).clone()
                    {
                        if self.globals.contains_key(&*name.as_string().unwrap()) {
                            self.globals
                                .insert(name.clone().to_string(), self.peek(0).clone());
                        } else {
                            return Err(RuntimeError {
                                message: format!("Assigned to undeclared global {:?}", name),
                                span: self.bin.spans[self.ip - 2],
                            });
                        }
                    } else {
                        panic!("Invalid SetGlobal operand, references {:?}", self.peek(0))
                    }
                }
                Some(OpCode::SetLongGlobal) => {
                    let index = (self.read_byte() * u8::max_value() + self.read_byte()) as u16;
                    if let Value::Obj(name, ObjKind::String) = self.bin.get_constant(index).clone()
                    {
                        if self.globals.contains_key(&*name.as_string().unwrap()) {
                            self.globals
                                .insert(name.clone().to_string(), self.peek(0).clone());
                        } else {
                            return Err(RuntimeError {
                                message: format!("Assigned to undeclared global {:?}", name),
                                span: self.bin.spans[self.ip - 3],
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
                    if let Value::Obj(name, ObjKind::String) = self.bin.get_constant(index).clone()
                    {
                        self.globals
                            .insert(name.as_string().unwrap().clone(), Value::Nil);
                    } else {
                        panic!(
                            "Invalid SetLongGlobal operand, references {:?}",
                            self.peek(0)
                        )
                    }
                }
                Some(OpCode::DeclareLongGlobal) => {
                    let index = (self.read_byte() * u8::max_value() + self.read_byte()) as u16;
                    if let Value::Obj(name, ObjKind::String) = self.bin.get_constant(index).clone()
                    {
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
                            self.bin[self.ip], self.ip
                        ),
                        span: self.bin.spans[self.ip],
                    })
                }
            }
            if cfg!(feature = "disassemble") {
                self.print_stack(output_stream);
                writeln!(output_stream, " Globals: {:?}", self.globals).unwrap();
                writeln!(output_stream).unwrap();
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
                        span: self.bin.spans[self.ip - 1],
                    });
                }
            }
            OpCode::Add => match left {
                Value::Number(_) => match right {
                    Value::Number(_) => {}
                    _ => {
                        return Err(RuntimeError {
                            message: String::from("Cannot apply '+' to Number and Non-Number"),
                            span: self.bin.spans[self.ip - 1],
                        });
                    }
                },
                Value::Obj(_, ObjKind::String) => match right {
                    Value::Obj(_, ObjKind::String) => {}
                    _ => {
                        return Err(RuntimeError {
                            message: String::from("Cannot apply '+' to String and Non-String"),
                            span: self.bin.spans[self.ip - 1],
                        });
                    }
                },
                _ => {
                    return Err(RuntimeError {
                        message: String::from("Cannot apply '+' to non-numeric or non-string type"),
                        span: self.bin.spans[self.ip - 1],
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
        if self.ip >= self.bin.len() {
            panic!(
                "read_byte out of bounds. bin: {}, ip: {}",
                self.bin.name, self.ip
            );
        }
        self.ip += 1;
        self.bin[self.ip - 1]
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

    fn print_stack<W: Write>(&self, output_stream: &mut W) {
        write!(output_stream, " Stack: ").unwrap();
        for v in &self.stack {
            write!(output_stream, "[{:?}] ", v).unwrap();
        }
        writeln!(output_stream).unwrap();
    }
}
