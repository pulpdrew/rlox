use crate::error::RLoxError;
use crate::executable::Executable;
use crate::object::Obj;
use crate::object::ObjFunction;
use crate::object::ObjKind;
use crate::opcode::OpCode;
use crate::token::Span;
use crate::value::Value;

use std::collections::HashMap;
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
    base: usize,
    stack: Vec<Value>,
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
            base: 0,
            stack: Vec::new(),
            globals: HashMap::new(),
        }
    }

    pub fn interpret<W: Write>(
        &mut self,
        function: &ObjFunction,
        output_stream: &mut W,
    ) -> Result<(), RuntimeError> {
        while self.ip < function.bin.len() {
            if cfg!(feature = "disassemble") {
                function.bin.disassemble_instruction(self.ip);
            }

            match OpCode::from(self.read_u8(&function.bin)) {
                OpCode::Constant => {
                    let index = self.read_u8(&function.bin) as u16;
                    self.push(function.bin.get_constant(index).clone());
                }
                OpCode::LongConstant => {
                    let index = (self.read_u8(&function.bin) * u8::max_value()
                        + self.read_u8(&function.bin)) as u16;
                    self.push(function.bin.get_constant(index).clone());
                }
                OpCode::Negate => {
                    if self.peek(0).is_number() {
                        let value = -self.peek(0).clone();
                        self.pop();
                        self.push(value);
                    } else {
                        return Err(RuntimeError {
                            message: String::from("Cannot negate non-numeric types"),
                            span: function.bin.spans[self.ip - 1],
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
                OpCode::Return => {
                    self.stack[self.base] = self.peek(0).clone();
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
                | op @ OpCode::Equal => match self.binary_op(&op, &function.bin) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                },
                OpCode::Print => {
                    writeln!(output_stream, "{:}", self.pop()).unwrap();
                    output_stream.flush().unwrap();
                }
                OpCode::GetGlobal => {
                    let index = self.read_u8(&function.bin) as u16;
                    let name_arg = function.bin.get_constant(index).clone();
                    if let Value::Obj(name, ObjKind::String) = name_arg {
                        let var_value = match self.globals.get(&*name.as_string().unwrap()) {
                            Some(value) => value.clone(),
                            None => {
                                return Err(RuntimeError {
                                    message: format!("Attempted to get unknown global {}", name),
                                    span: function.bin.spans[self.ip - 1],
                                })
                            }
                        };
                        self.push(var_value);
                    } else {
                        panic!("Attempt to assign to global {:?}", self.peek(0))
                    }
                }
                OpCode::GetLongGlobal => {
                    let index = (self.read_u8(&function.bin) * u8::max_value()
                        + self.read_u8(&function.bin)) as u16;
                    let name_arg = function.bin.get_constant(index).clone();
                    if let Value::Obj(name, ObjKind::String) = name_arg {
                        let var_value = match self.globals.get(&*name.as_string().unwrap()) {
                            Some(value) => value.clone(),
                            None => {
                                return Err(RuntimeError {
                                    message: format!("Attempted to get unknown global {}", name),
                                    span: function.bin.spans[self.ip - 2],
                                })
                            }
                        };
                        self.push(var_value);
                    } else {
                        panic!("Attempt to assign to global {:?}", self.peek(0))
                    }
                }
                OpCode::SetGlobal => {
                    let index = self.read_u8(&function.bin) as u16;
                    if let Value::Obj(name, ObjKind::String) =
                        function.bin.get_constant(index).clone()
                    {
                        if self.globals.contains_key(&*name.as_string().unwrap()) {
                            self.globals
                                .insert(name.clone().to_string(), self.peek(0).clone());
                        } else {
                            return Err(RuntimeError {
                                message: format!("Assigned to set undeclared global {}", name),
                                span: function.bin.spans[self.ip - 1],
                            });
                        }
                    } else {
                        panic!("Invalid SetGlobal operand, references {:?}", self.peek(0))
                    }
                }
                OpCode::SetLongGlobal => {
                    let index = (self.read_u8(&function.bin) * u8::max_value()
                        + self.read_u8(&function.bin)) as u16;
                    if let Value::Obj(name, ObjKind::String) =
                        function.bin.get_constant(index).clone()
                    {
                        if self.globals.contains_key(&*name.as_string().unwrap()) {
                            self.globals
                                .insert(name.clone().to_string(), self.peek(0).clone());
                        } else {
                            return Err(RuntimeError {
                                message: format!("Assigned to set undeclared global {}", name),
                                span: function.bin.spans[self.ip - 2],
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
                    let index = self.read_u8(&function.bin) as u16;
                    if let Value::Obj(name, ObjKind::String) =
                        &function.bin.get_constant(index).clone()
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
                    let index = (self.read_u8(&function.bin) * u8::max_value()
                        + self.read_u8(&function.bin)) as u16;
                    if let Value::Obj(name, ObjKind::String) =
                        &function.bin.get_constant(index).clone()
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
                    let index = self.read_u8(&function.bin) as usize;
                    self.push(self.stack[self.base + index].clone());
                }
                OpCode::SetLocal => {
                    let index = self.read_u8(&function.bin) as usize;
                    let stack_len = self.stack.len();
                    self.stack[stack_len - 2 - index] = self.peek(0).clone();
                }
                OpCode::Jump => {
                    let destination = self.read_u16(&function.bin);
                    self.ip = destination as usize;
                }
                OpCode::JumpIfTrue => {
                    let destination = self.read_u16(&function.bin);
                    if self.peek(0).is_truthy() {
                        self.ip = destination as usize;
                    }
                }
                OpCode::JumpIfFalse => {
                    let destination = self.read_u16(&function.bin);
                    if !self.peek(0).is_truthy() {
                        self.ip = destination as usize;
                    }
                }
                OpCode::Call => {
                    let arg_count = self.read_u8(&function.bin);
                    let callable = self.peek(arg_count as usize).clone();

                    match callable {
                        Value::Obj(function, ObjKind::Function) => {
                            if let Obj::Function(f) = &*function {
                                self.call(f, arg_count, output_stream)?;
                            }
                        }
                        _ => {
                            return Err(RuntimeError {
                                message: format!("Cannot call {}", callable),
                                span: function.bin.spans[self.ip - 1],
                            });
                        }
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
        function: &ObjFunction,
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
        self.interpret(function, output_stream)?;

        // Remove everything from the stack except the return value
        for _ in (self.base + 1)..self.stack.len() {
            self.pop();
        }

        // Restore the ip and the base
        self.ip = ip_backup;
        self.base = base_backup;

        Ok(())
    }

    fn binary_op(&mut self, op: &OpCode, bin: &Executable) -> Result<(), RuntimeError> {
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
                        span: bin.spans[self.ip - 1],
                    });
                }
            }
            OpCode::Add => match left {
                Value::Number(_) => match right {
                    Value::Number(_) => {}
                    _ => {
                        return Err(RuntimeError {
                            message: String::from("Cannot apply '+' to Number and Non-Number"),
                            span: bin.spans[self.ip - 1],
                        });
                    }
                },
                Value::Obj(_, ObjKind::String) => match right {
                    Value::Obj(_, ObjKind::String) => {}
                    _ => {
                        return Err(RuntimeError {
                            message: String::from("Cannot apply '+' to String and Non-String"),
                            span: bin.spans[self.ip - 1],
                        });
                    }
                },
                _ => {
                    return Err(RuntimeError {
                        message: String::from("Cannot apply '+' to non-numeric or non-string type"),
                        span: bin.spans[self.ip - 1],
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

    fn read_u8(&mut self, bin: &Executable) -> u8 {
        if self.ip >= bin.len() {
            panic!("read_u8 out of bounds. bin: {}, ip: {}", bin.name, self.ip);
        }
        self.ip += 1;
        bin.read_u8(self.ip - 1)
    }

    fn read_u16(&mut self, bin: &Executable) -> u16 {
        if self.ip >= bin.len() {
            panic!("read_u16 out of bounds. bin: {}, ip: {}", bin.name, self.ip);
        }

        self.ip += 2;
        bin.read_u16(self.ip - 2)
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("Popped an empty stack")
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack.len() - distance - 1]
    }

    fn print_stack<W: Write>(&self, output_stream: &mut W) {
        write!(output_stream, " Stack: ").unwrap();
        for (index, value) in self.stack.iter().enumerate() {
            if index == self.base {
                write!(output_stream, ">< ").unwrap();
            }
            write!(output_stream, "[{:?}] ", value).unwrap();
        }
        writeln!(output_stream).unwrap();
    }
}
