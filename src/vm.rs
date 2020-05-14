use crate::error::ReportableError;
use crate::executable::Executable;
use crate::object::{ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjUpvalue};
use crate::opcode::OpCode;
use crate::token::Span;
use crate::value::Value;

use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
    span: Span,
}

impl ReportableError for RuntimeError {
    fn span(&self) -> Span {
        self.span
    }
    fn message(&self) -> String {
        format!("Runtime Error - {}", self.message)
    }
}

#[derive(Debug, Default)]
pub struct VM {
    ip: usize,
    base: usize,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            ip: 0,
            base: 0,
            stack: Vec::new(),
            globals: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.ip = 0;
        self.base = 0;
        self.stack = Vec::new();
    }

    pub fn interpret<W: Write>(
        &mut self,
        closure: &ObjClosure,
        output_stream: &mut W,
    ) -> Result<(), RuntimeError> {
        while self.ip < closure.function.bin.len() {
            if cfg!(feature = "disassemble") {
                closure
                    .function
                    .bin
                    .disassemble_instruction(self.ip, output_stream);
            }

            match OpCode::from(self.read_u8(&closure.function.bin)?) {
                OpCode::Constant => {
                    let index = self.read_u8(&closure.function.bin)? as u16;
                    self.push(closure.function.bin.get_constant(index).clone());
                }
                OpCode::LongConstant => {
                    let index = self.read_u16(&closure.function.bin)?;
                    self.push(closure.function.bin.get_constant(index).clone());
                }
                OpCode::Negate => {
                    if self.peek(0)?.is_number() {
                        let value = -self.peek(0)?.clone();
                        self.pop()?;
                        self.push(value);
                    } else {
                        return Err(RuntimeError {
                            message: String::from("Cannot negate non-numeric types"),
                            span: closure.function.bin.spans[self.ip - 1],
                        });
                    }
                }
                OpCode::Pop => {
                    self.pop()?;
                }
                OpCode::Not => {
                    let right = self.peek(0);
                    let value = Value::Bool(!right?.is_truthy());
                    self.pop()?;
                    self.push(value);
                }
                OpCode::Return => {
                    self.stack[self.base] = self.peek(0)?.clone();
                    return Ok(());
                }
                op @ OpCode::Add
                | op @ OpCode::Subtract
                | op @ OpCode::Multiply
                | op @ OpCode::Divide
                | op @ OpCode::Less
                | op @ OpCode::LessEqual
                | op @ OpCode::Greater
                | op @ OpCode::GreaterEqual
                | op @ OpCode::Equal => match self.binary_op(&op, &closure.function.bin) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                },
                OpCode::Print => {
                    writeln!(output_stream, "{:}", self.pop()?).unwrap();
                    output_stream.flush().unwrap();
                }
                OpCode::GetGlobal => {
                    let index = self.read_u8(&closure.function.bin)? as u16;
                    self.get_global(index, &*closure.function)?;
                }
                OpCode::GetLongGlobal => {
                    let index = self.read_u16(&closure.function.bin)?;
                    self.get_global(index, &*closure.function)?;
                }
                OpCode::SetGlobal => {
                    let index = self.read_u8(&closure.function.bin)? as u16;
                    self.set_global(index, &*closure.function)?;
                }
                OpCode::SetLongGlobal => {
                    let index = self.read_u16(&closure.function.bin)?;
                    self.set_global(index, &*closure.function)?;
                }
                OpCode::DeclareGlobal => {
                    let index = self.read_u8(&closure.function.bin)? as u16;
                    self.declare_global(index, &*closure.function)?;
                }
                OpCode::DeclareLongGlobal => {
                    let index = self.read_u16(&closure.function.bin)?;
                    self.declare_global(index, &*closure.function)?;
                }
                OpCode::GetLocal => {
                    let index = self.read_u8(&closure.function.bin)? as usize;
                    self.push(self.stack[self.base + index].clone());
                }
                OpCode::SetLocal => {
                    let index = self.read_u8(&closure.function.bin)? as usize;
                    let stack_len = self.stack.len();
                    self.stack[stack_len - 2 - index] = self.peek(0)?.clone();
                }
                OpCode::Jump => {
                    let destination = self.read_u16(&closure.function.bin)?;
                    self.ip = destination as usize;
                }
                OpCode::JumpIfTrue => {
                    let destination = self.read_u16(&closure.function.bin)?;
                    if self.peek(0)?.is_truthy() {
                        self.ip = destination as usize;
                    }
                }
                OpCode::JumpIfFalse => {
                    let destination = self.read_u16(&closure.function.bin)?;
                    if !self.peek(0)?.is_truthy() {
                        self.ip = destination as usize;
                    }
                }
                OpCode::Invoke => {
                    let arg_count = self.read_u8(&closure.function.bin)?;
                    let callable = self.peek(arg_count as usize)?.clone();

                    match callable {
                        Value::Closure(closure) => {
                            self.call(&*closure, arg_count, output_stream)?;
                        }
                        Value::Class(class) => {
                            self.instantiate(&class, arg_count, output_stream)?;
                        }
                        _ => {
                            return Err(RuntimeError {
                                message: format!("Cannot call {}", callable),
                                span: closure.function.bin.spans[self.ip - 1],
                            });
                        }
                    }
                }
                OpCode::Closure => {
                    let index = self.read_u8(&closure.function.bin)? as u16;
                    let arg_value = closure.function.bin.get_constant(index).clone();

                    let function = if let Value::Function(f) = arg_value {
                        f.clone()
                    } else {
                        return Err(RuntimeError {
                            message: format!("Closure instruction expected function constant argument, but got {}", arg_value),
                            span: closure.function.bin.spans[self.ip - 1]
                        });
                    };

                    let upvalues = function
                        .upvalues
                        .iter()
                        .map(|(is_local, index)| {
                            if *is_local {
                                ObjUpvalue::from(self.stack[self.base + index].clone())
                            } else {
                                ObjUpvalue::from(closure.upvalues[*index].value.clone())
                            }
                        })
                        .collect();

                    let closure = ObjClosure {
                        function: function,
                        upvalues,
                    };
                    let closure_value = Value::from(closure);
                    self.push(closure_value);
                }
                OpCode::GetUpvalue => {
                    let index = self.read_u8(&closure.function.bin)?;
                    self.push(closure.upvalues[index as usize].value.clone());
                }
                OpCode::ReadField => {
                    let name_index = self.read_u8(&closure.function.bin)?;
                    let name_constant = closure.function.bin.get_constant(name_index as u16);

                    let name = if let Value::String(s) = name_constant {
                        &s.string
                    } else {
                        return Err(RuntimeError {
                            message: format!(
                                "Expected field name ObjString but found {:?}",
                                name_constant
                            ),
                            span: closure.function.bin.spans[self.ip - 2],
                        });
                    };

                    let target_value = self.peek(0)?.clone();
                    if let Value::Instance(instance) = target_value {
                        if let Some(v) = instance.borrow().fields.get(name) {
                            self.push(v.clone());
                        } else {
                            return Err(RuntimeError {
                                message: format!("{:?} has no field {}", instance, name),
                                span: closure.function.bin.spans[self.ip - 1],
                            });
                        }
                    } else {
                        return Err(RuntimeError {
                            message: format!("{:?} is not an instance", target_value),
                            span: closure.function.bin.spans[self.ip - 1],
                        });
                    }
                }
                OpCode::SetField => {
                    let name_index = self.read_u8(&closure.function.bin)?;
                    let name_constant = closure.function.bin.get_constant(name_index as u16);

                    let field_name = if let Value::String(s) = name_constant {
                        &s.string
                    } else {
                        return Err(RuntimeError {
                            message: format!(
                                "Expected field name ObjString but found {:?}",
                                name_constant
                            ),
                            span: closure.function.bin.spans[self.ip - 2],
                        });
                    };

                    let target_value = self.peek(1)?.clone();
                    if let Value::Instance(instance) = target_value {
                        let rvalue = self.peek(0)?.clone();
                        instance
                            .borrow_mut()
                            .fields
                            .insert(field_name.clone(), rvalue);
                    } else {
                        return Err(RuntimeError {
                            message: format!("{:?} is not an instance", target_value),
                            span: closure.function.bin.spans[self.ip - 1],
                        });
                    }
                }
            }
            if cfg!(feature = "disassemble") {
                self.print_stack(output_stream);
                writeln!(output_stream, " Globals: {:?}", self.globals).unwrap();
                writeln!(output_stream).unwrap();
            }
        }

        Ok(())
    }

    fn call<W: Write>(
        &mut self,
        closure: &ObjClosure,
        arg_count: u8,
        output_stream: &mut W,
    ) -> Result<(), RuntimeError> {
        // Save the current IP and base to restore after returning
        let ip_backup = self.ip;
        let base_backup = self.base;

        // The arguments should already be on the stack.
        // Adjust the base pointer to point at their start
        self.base = self.stack.len() - arg_count as usize;

        // Execution should begin at the beginning of the function
        self.ip = 0;

        // Run the function
        self.interpret(closure, output_stream)?;

        // Remove everything from the stack except the return value
        for _ in (self.base + 1)..self.stack.len() {
            self.pop()?;
        }

        // Restore the ip and the base
        self.ip = ip_backup;
        self.base = base_backup;

        Ok(())
    }

    fn instantiate<W: Write>(
        &mut self,
        class: &Rc<ObjClass>,
        _arg_count: u8,
        _output_stream: &mut W,
    ) -> Result<(), RuntimeError> {
        let instance = ObjInstance::from(class);
        self.push(Value::from(instance));
        Ok(())
    }

    fn binary_op(&mut self, op: &OpCode, bin: &Executable) -> Result<(), RuntimeError> {
        let right = self.peek(0)?.clone();
        let left = self.peek(1)?.clone();

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
                        message: format!("Cannot apply '{:?}' to non-numeric types", op),
                        span: bin.spans[self.ip - 1],
                    });
                }
            }
            OpCode::Add => {
                if left.is_number() && !right.is_number() {
                    return Err(RuntimeError {
                        message: String::from("Cannot apply '+' to Number and Non-Number"),
                        span: bin.spans[self.ip - 1],
                    });
                } else if left.is_string() && !right.is_string() {
                    return Err(RuntimeError {
                        message: String::from("Cannot apply '+' to String and Non-String"),
                        span: bin.spans[self.ip - 1],
                    });
                } else if !left.is_number() && !left.is_string() {
                    return Err(RuntimeError {
                        message: String::from("Cannot apply '+' to non-numeric or non-string type"),
                        span: bin.spans[self.ip - 1],
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
            _ => {
                return Err(RuntimeError {
                    message: format!("Invalid binary operation {:?}", op),
                    span: bin.spans[self.ip - 1],
                })
            }
        };
        self.pop()?;
        self.pop()?;
        self.push(value);
        Ok(())
    }

    fn get_global(&mut self, name_index: u16, function: &ObjFunction) -> Result<(), RuntimeError> {
        let name_arg = function.bin.get_constant(name_index);
        let value = if let Value::String(name) = name_arg {
            if let Some(value) = self.globals.get(&name.string) {
                value.clone()
            } else {
                return Err(RuntimeError {
                    message: format!("Attempted to get unknown global {}", name),
                    span: function.bin.spans[self.ip - 1],
                });
            }
        } else {
            return Err(RuntimeError {
                message: format!(
                    "Attempted to lookup global by non-string name {:?}",
                    name_arg
                ),
                span: function.bin.spans[self.ip - 1],
            });
        };
        self.push(value);
        Ok(())
    }

    fn set_global(&mut self, name_index: u16, function: &ObjFunction) -> Result<(), RuntimeError> {
        let name_arg = function.bin.get_constant(name_index);
        if let Value::String(name) = name_arg {
            if self.globals.contains_key(&name.string) {
                self.globals
                    .insert(name.string.clone(), self.peek(0)?.clone());
            } else {
                return Err(RuntimeError {
                    message: format!("Assigned to set undeclared global {}", name),
                    span: function.bin.spans[self.ip - 1],
                });
            }
        } else {
            return Err(RuntimeError {
                message: format!("Attempted to set global by non-string name {:?}", name_arg),
                span: function.bin.spans[self.ip - 1],
            });
        }

        Ok(())
    }

    fn declare_global(
        &mut self,
        name_index: u16,
        function: &ObjFunction,
    ) -> Result<(), RuntimeError> {
        let name_arg = function.bin.get_constant(name_index);
        if let Value::String(name) = name_arg {
            self.globals.insert(name.string.clone(), Value::Nil);
        } else {
            return Err(RuntimeError {
                message: format!(
                    "Attempted to declare global by non-string name {:?}",
                    name_arg
                ),
                span: function.bin.spans[self.ip - 1],
            });
        }

        Ok(())
    }

    fn read_u8(&mut self, bin: &Executable) -> Result<u8, RuntimeError> {
        if self.ip >= bin.len() {
            Err(RuntimeError {
                message: format!("read_u8 out of bounds. bin: {}, ip: {}", bin.name, self.ip),
                span: bin.spans[self.ip - 1],
            })
        } else {
            self.ip += 1;
            Ok(bin.read_u8(self.ip - 1))
        }
    }

    fn read_u16(&mut self, bin: &Executable) -> Result<u16, RuntimeError> {
        if self.ip + 1 >= bin.len() {
            Err(RuntimeError {
                message: format!("read_u16 out of bounds. bin: {}, ip: {}", bin.name, self.ip),
                span: bin.spans[self.ip - 2],
            })
        } else {
            self.ip += 2;
            Ok(bin.read_u16(self.ip - 2))
        }
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Result<Value, RuntimeError> {
        match self.stack.pop() {
            Some(v) => Ok(v),
            None => Err(RuntimeError {
                message: "Attempted pop() on an empty stack".to_string(),
                span: Span::new(0, 0),
            }),
        }
    }

    fn peek(&self, distance: usize) -> Result<&Value, RuntimeError> {
        if self.stack.len() <= distance {
            Err(RuntimeError {
                message: format!(
                    "Attempted to peek({}) but stack length is {}.",
                    distance,
                    self.stack.len()
                ),
                span: Span::new(0, 0),
            })
        } else {
            Ok(&self.stack[self.stack.len() - distance - 1])
        }
    }

    fn print_stack<W: Write>(&self, output_stream: &mut W) {
        write!(output_stream, " Stack: ").unwrap();
        for (index, value) in self.stack.iter().enumerate() {
            if index == self.base {
                write!(output_stream, "^ ").unwrap();
            }
            write!(output_stream, "[{:?}] ", value).unwrap();
        }
        writeln!(output_stream).unwrap();
    }
}
