use crate::error::RLoxError;
use crate::executable::Executable;
use crate::object::ObjKind;
use crate::opcode::OpCode;
use crate::token::Span;
use crate::value::Value;

use std::collections::{HashMap, VecDeque};
use std::io::Write;

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
    span: Span,
}

impl RLoxError for RuntimeError {
    fn span(&self) -> Span {
        self.span
    }
    fn message(&self) -> String {
        format!("Runtime Error - {}", self.message)
    }
}

#[derive(Debug)]
pub struct VM {
    ip: usize,
    bin: Executable,
    stack: VecDeque<Value>,
    globals: HashMap<String, Value>,
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

            match OpCode::from(self.read_u8()) {
                OpCode::Constant => {
                    let index = self.read_u8() as u16;
                    self.push(self.bin.get_constant(index).clone());
                }
                OpCode::LongConstant => {
                    let index = (self.read_u8() * u8::max_value() + self.read_u8()) as u16;
                    self.push(self.bin.get_constant(index).clone());
                }
                OpCode::Negate => {
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
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::Not => {
                    let right = self.peek(0);
                    let value = Value::Bool(!right.is_truthy());
                    self.pop();
                    self.push(value);
                }
                OpCode::Return => return Ok(()),
                op @ OpCode::Add
                | op @ OpCode::Subtract
                | op @ OpCode::Multiply
                | op @ OpCode::Divide
                | op @ OpCode::Less
                | op @ OpCode::LessEqual
                | op @ OpCode::Greater
                | op @ OpCode::GreaterEqual
                | op @ OpCode::Equal => match self.binary_op(&op) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                },
                OpCode::Print => {
                    writeln!(output_stream, "{:}", self.pop()).unwrap();
                    output_stream.flush().unwrap();
                }
                OpCode::GetGlobal => {
                    let index = self.read_u8() as u16;
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
                OpCode::GetLongGlobal => {
                    let index = (self.read_u8() * u8::max_value() + self.read_u8()) as u16;
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
                OpCode::SetGlobal => {
                    let index = self.read_u8() as u16;
                    if let Value::Obj(name, ObjKind::String) = self.bin.get_constant(index).clone()
                    {
                        if self.globals.contains_key(&*name.as_string().unwrap()) {
                            self.globals
                                .insert(name.clone().to_string(), self.peek(0).clone());
                        } else {
                            return Err(RuntimeError {
                                message: format!("Assigned to undeclared global {:}", name),
                                span: self.bin.spans[self.ip - 2],
                            });
                        }
                    } else {
                        panic!("Invalid SetGlobal operand, references {:?}", self.peek(0))
                    }
                }
                OpCode::SetLongGlobal => {
                    let index = (self.read_u8() * u8::max_value() + self.read_u8()) as u16;
                    if let Value::Obj(name, ObjKind::String) = self.bin.get_constant(index).clone()
                    {
                        if self.globals.contains_key(&*name.as_string().unwrap()) {
                            self.globals
                                .insert(name.clone().to_string(), self.peek(0).clone());
                        } else {
                            return Err(RuntimeError {
                                message: format!("Assigned to undeclared global {:}", name),
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
                OpCode::DeclareGlobal => {
                    let index = self.read_u8() as u16;
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
                OpCode::DeclareLongGlobal => {
                    let index = (self.read_u8() * u8::max_value() + self.read_u8()) as u16;
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
                OpCode::GetLocal => {
                    let index = self.read_u8() as usize;
                    self.push(self.peek(index).clone());
                }
                OpCode::SetLocal => {
                    let index = self.read_u8() as usize;
                    let stack_len = self.stack.len();
                    self.stack[stack_len - 2 - index] = self.peek(0).clone();
                }
                OpCode::Jump => {
                    let destination = self.read_u16();
                    self.ip = destination as usize;
                }
                OpCode::JumpIfTrue => {
                    let destination = self.read_u16();
                    if self.peek(0).is_truthy() {
                        self.ip = destination as usize;
                    }
                }
                OpCode::JumpIfFalse => {
                    let destination = self.read_u16();
                    if !self.peek(0).is_truthy() {
                        self.ip = destination as usize;
                    }
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

    fn read_u8(&mut self) -> u8 {
        if self.ip >= self.bin.len() {
            panic!(
                "read_u8 out of bounds. bin: {}, ip: {}",
                self.bin.name, self.ip
            );
        }
        self.ip += 1;
        self.bin[self.ip - 1]
    }

    fn read_u16(&mut self) -> u16 {
        if self.ip >= self.bin.len() {
            panic!(
                "read_u16 out of bounds. bin: {}, ip: {}",
                self.bin.name, self.ip
            );
        }
        let high = self.bin[self.ip];
        let low = self.bin[self.ip + 1];

        self.ip += 2;
        ((high as u16) << 8) + low as u16
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
