use crate::error::RuntimeError;
use crate::executable::Executable;
use crate::object::{ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjUpvalue};
use crate::opcode::OpCode;
use crate::token::Span;
use crate::value::Value;

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct VM {
    /// The index of the next byte to be read from the executable
    ip: usize,

    /// The index in `stack` that is the bottom of the current frame
    base: usize,

    /// The runtime value stack
    stack: Vec<Value>,

    /// The current global variables
    globals: HashMap<String, Value>,
}

impl VM {
    /// Create a new, empty VM
    pub fn new() -> Self {
        VM {
            ip: 0,
            base: 0,
            stack: Vec::new(),
            globals: HashMap::new(),
        }
    }

    /// Reset the VM's state, keeping the global variables
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
            let op = closure.function.bin[self.ip];
            self.ip += 1;

            if cfg!(feature = "disassemble") {
                writeln!(output_stream, "{:?}", op).unwrap();
            }
            match op {
                OpCode::Constant(index) => {
                    self.push(closure.function.bin.get_constant(index).clone());
                }
                OpCode::Negate => {
                    let argument = self.pop()?;
                    argument.assert_is_number_or(
                        "Cannot negate non-numeric types",
                        closure.function.bin.spans[self.ip - 1],
                    )?;
                    self.push(-argument);
                }
                OpCode::Pop => {
                    self.pop()?;
                }
                OpCode::Not => {
                    let argument = self.pop()?;
                    self.push(Value::from(!argument.is_truthy()));
                }
                OpCode::Return => {
                    self.stack[self.base] = self.peek(0)?.clone();
                    return Ok(());
                }
                OpCode::Add
                | OpCode::Subtract
                | OpCode::Multiply
                | OpCode::Divide
                | OpCode::Less
                | OpCode::LessEqual
                | OpCode::Greater
                | OpCode::GreaterEqual
                | OpCode::Equal
                | OpCode::NotEqual => match self.binary_op(&op, &closure.function.bin) {
                    Ok(()) => {}
                    Err(e) => return Err(e),
                },
                OpCode::Print => {
                    writeln!(output_stream, "{:}", self.pop()?).unwrap();
                    output_stream.flush().unwrap();
                }
                OpCode::GetGlobal(name_index) => {
                    self.get_global(name_index, &*closure.function)?;
                }
                OpCode::SetGlobal(name_index) => {
                    self.set_global(name_index, &*closure.function)?;
                }
                OpCode::DeclareGlobal(name_index) => {
                    self.declare_global(name_index, &*closure.function)?;
                }
                OpCode::GetLocal(index) => {
                    self.push(self.stack[self.base + index].clone());
                }
                OpCode::SetLocal(index) => {
                    let stack_len = self.stack.len();
                    self.stack[stack_len - 2 - index] = self.peek(0)?.clone();
                }
                OpCode::Jump(destination) => {
                    self.ip = destination as usize;
                }
                OpCode::JumpIfTrue(destination) => {
                    if self.peek(0)?.is_truthy() {
                        self.ip = destination as usize;
                    }
                }
                OpCode::JumpIfFalse(destination) => {
                    if !self.peek(0)?.is_truthy() {
                        self.ip = destination as usize;
                    }
                }
                OpCode::Invoke(arg_count) => {
                    let callable = self.peek(arg_count + 1)?.clone();

                    match callable {
                        Value::Closure(closure) => {
                            self.call(&*closure, arg_count, output_stream)?;
                        }
                        Value::BoundMethod(method) => {
                            let stack_len = self.stack.len();
                            self.stack[stack_len - (arg_count + 1) as usize] =
                                Value::Instance(method.receiver.clone());
                            self.call(&*method.method, arg_count, output_stream)?;
                        }
                        Value::Class(class) => {
                            self.instantiate(&class, arg_count, output_stream)?;
                        }
                        _ => {
                            return Err(RuntimeError {
                                message: format!("Cannot invoke {}", callable),
                                span: closure.function.bin.spans[self.ip - 1],
                            });
                        }
                    }
                }
                OpCode::Closure(index) => {
                    let arg_value = closure.function.bin.get_constant(index).clone();

                    let function = if let Value::Function(f) = arg_value {
                        f.clone()
                    } else {
                        return Err(RuntimeError {
                            message: format!("Closure instruction expected function constant argument, but got {}", arg_value),
                            span: closure.function.bin.spans[self.ip - 1]
                        });
                    };

                    let upvalues = RefCell::new(
                        function
                            .upvalues
                            .iter()
                            .map(|(is_local, index)| {
                                if *is_local {
                                    ObjUpvalue::from(self.stack[self.base + index].clone())
                                } else {
                                    ObjUpvalue::from(
                                        closure
                                            .upvalues
                                            .borrow()
                                            .get(*index)
                                            .unwrap()
                                            .value
                                            .clone(),
                                    )
                                }
                            })
                            .collect(),
                    );

                    let closure = ObjClosure { function, upvalues };
                    let closure_value = Value::from(closure);
                    self.push(closure_value);
                }
                OpCode::GetUpvalue(index) => {
                    self.push(closure.upvalues.borrow().get(index).unwrap().value.clone());
                }
                OpCode::ReadField(name_index) => {
                    let name_constant = closure.function.bin.get_constant(name_index);

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

                    let target_value = self.pop()?;
                    if let Value::Instance(instance) = target_value {
                        if let Some(method) = instance.class.methods.borrow().get(name) {
                            self.push(Value::BoundMethod(Rc::new(ObjBoundMethod {
                                receiver: instance.clone(),
                                method: method.clone(),
                            })));
                        } else if let Some(v) = instance.fields.borrow().get(name) {
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
                OpCode::SetField(name_index) => {
                    let name_constant = closure.function.bin.get_constant(name_index);

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

                    let rvalue = self.pop()?;
                    let target_value = self.pop()?;
                    if let Value::Instance(instance) = target_value {
                        instance
                            .fields
                            .borrow_mut()
                            .insert(field_name.clone(), rvalue.clone());
                        self.push(rvalue);
                    } else {
                        return Err(RuntimeError {
                            message: format!("{:?} is not an instance", target_value),
                            span: closure.function.bin.spans[self.ip - 1],
                        });
                    }
                }
                OpCode::SetUpvalue(index) => closure
                    .upvalues
                    .borrow_mut()
                    .insert(index, ObjUpvalue::from(self.peek(0)?.clone())),
                OpCode::Method => {
                    let method_closure = self.pop()?.unwrap_closure_or(
                        "Expected a closure value at the top of the stack",
                        closure.function.bin.spans[self.ip - 1],
                    )?;

                    let class = self.peek(0)?.unwrap_class_or(
                        "Expected a class value at stack[top - 1]",
                        closure.function.bin.spans[self.ip - 1],
                    )?;

                    class.methods.borrow_mut().insert(
                        method_closure.function.name.string.clone(),
                        method_closure.clone(),
                    );
                }
                OpCode::Inherit => {
                    let superclass = self.peek(1)?.unwrap_class_or(
                        "Cannot inherit from a non-class value",
                        closure.function.bin.spans[self.ip - 1],
                    )?;
                    let class = self.peek(0)?.unwrap_class_or(
                        "Cannot inherit into a non-class value",
                        closure.function.bin.spans[self.ip - 1],
                    )?;

                    for (method_name, method) in superclass.methods.borrow().iter() {
                        class
                            .methods
                            .borrow_mut()
                            .insert(method_name.clone(), method.clone());
                    }
                }
                OpCode::GetSuper(name_index) => {
                    if let Value::Class(class) = self.pop()? {
                        let method_name = closure.function.bin.get_constant(name_index);
                        let method_name = if let Value::String(string) = method_name {
                            &string.string
                        } else {
                            return Err(RuntimeError {
                                message: format!(
                                    "Expected string constant argument but got {}",
                                    method_name
                                ),
                                span: closure.function.bin.spans[self.ip - 1],
                            });
                        };

                        if let Some(method) = class.methods.borrow().get(method_name) {
                            if let Value::Instance(instance) = self.pop()? {
                                self.push(Value::BoundMethod(Rc::new(ObjBoundMethod {
                                    receiver: instance.clone(),
                                    method: method.clone(),
                                })));
                            } else {
                                return Err(RuntimeError {
                                    message: "expected receiver instance on the stack".to_string(),
                                    span: closure.function.bin.spans[self.ip - 1],
                                });
                            }
                        } else {
                            return Err(RuntimeError {
                                message: format!("'super' has no method {}", method_name),
                                span: closure.function.bin.spans[self.ip - 1],
                            });
                        }
                    } else {
                        return Err(RuntimeError {
                            message: "'super' is not a class".to_string(),
                            span: closure.function.bin.spans[self.ip - 1],
                        });
                    }
                }
                OpCode::Bool => {
                    let truthiness = self.pop()?.is_truthy();
                    self.push(truthiness.into())
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
        arg_count: usize,
        output_stream: &mut W,
    ) -> Result<(), RuntimeError> {
        // Save the current IP and base to restore after returning
        let ip_backup = self.ip;
        let base_backup = self.base;

        // The arguments should already be on the stack.
        // Adjust the base pointer to point at their start
        self.base = self.stack.len() - (arg_count + 1) as usize;

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
        arg_count: usize,
        output_stream: &mut W,
    ) -> Result<(), RuntimeError> {
        // Create a new instance
        let instance = ObjInstance::from(class);
        let instance_value = Value::from(instance);

        // Run the init method if there is one
        if class.methods.borrow().contains_key("init") {
            // Use the new instance as "this"
            let stack_len = self.stack.len();
            self.stack[stack_len - (arg_count + 1) as usize] = instance_value.clone();

            self.call(
                &class.methods.borrow_mut().get("init").unwrap(),
                arg_count,
                output_stream,
            )?;

            // Ignore any return value
            self.pop()?;
        }

        // Pop the class (callable)
        self.pop()?;

        // Leave the new instance on the top of the stack
        self.push(instance_value);

        Ok(())
    }

    fn binary_op(&mut self, op: &OpCode, bin: &Executable) -> Result<(), RuntimeError> {
        let right = self.pop()?;
        let left = self.pop()?;

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
            OpCode::NotEqual => Value::Bool(left != right),
            _ => {
                return Err(RuntimeError {
                    message: format!("Invalid binary operation {:?}", op),
                    span: bin.spans[self.ip - 1],
                })
            }
        };
        self.push(value);
        Ok(())
    }

    fn get_global(
        &mut self,
        name_index: usize,
        function: &ObjFunction,
    ) -> Result<(), RuntimeError> {
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

    fn set_global(
        &mut self,
        name_index: usize,
        function: &ObjFunction,
    ) -> Result<(), RuntimeError> {
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
        name_index: usize,
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

impl Value {
    /// Returns an error with the given message and span if the value is not a Number variant
    fn assert_is_number_or(&self, message: &str, span: Span) -> Result<(), RuntimeError> {
        if self.is_number() {
            Ok(())
        } else {
            Err(RuntimeError {
                message: message.to_string(),
                span,
            })
        }
    }
    /// Unwraps a `Closure` variant from the `Value` or returns an error with the given message and span
    fn unwrap_closure_or(&self, message: &str, span: Span) -> Result<Rc<ObjClosure>, RuntimeError> {
        if let Value::Closure(closure) = self {
            Ok(closure.clone())
        } else {
            Err(RuntimeError {
                message: message.to_string(),
                span,
            })
        }
    }
    /// Unwraps a `Class` variant from the `Value` or returns an error with the given message and span
    fn unwrap_class_or(&self, message: &str, span: Span) -> Result<Rc<ObjClass>, RuntimeError> {
        if let Value::Class(class) = self {
            Ok(class.clone())
        } else {
            Err(RuntimeError {
                message: message.to_string(),
                span,
            })
        }
    }
}
