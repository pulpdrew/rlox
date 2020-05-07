use crate::ast::{AstNode, Expression, Statement};
use crate::error::ReportableError;
use crate::executable::Executable;
use crate::object::{Obj, ObjFunction};
use crate::opcode::OpCode;
use crate::token::{Kind, Span};
use crate::value::Value;
use std::io::Write;

#[derive(Debug)]
pub struct CompilationError {
    message: String,
    span: Span,
}

impl ReportableError for CompilationError {
    fn span(&self) -> Span {
        self.span
    }
    fn message(&self) -> String {
        format!("Runtime Error - {}", self.message)
    }
}

#[derive(Debug)]
pub struct Compiler<'a, W: Write> {
    function: Option<Obj>,
    locals: Vec<Local>,
    scope_depth: usize,
    output_stream: &'a mut W,
}

#[derive(Debug)]
struct Local {
    name: String,
    depth: usize,
}

pub fn compile<W: Write>(
    program: Vec<AstNode>,
    output_stream: &mut W,
) -> Result<ObjFunction, CompilationError> {
    let mut compiler = Compiler::new(output_stream);
    let mut bin = Executable::new(String::from("script"));

    match compiler.compile_into(program, &mut bin) {
        Ok(..) => Ok(ObjFunction {
            arity: 0,
            bin,
            name: Box::new(Obj::from("script")),
        }),
        Err(e) => Err(e),
    }
}

impl<'a, W: Write> Compiler<'a, W> {
    pub fn new(output_stream: &'a mut W) -> Self {
        Compiler {
            function: None,
            locals: vec![],
            scope_depth: 0,
            output_stream,
        }
    }
    pub fn compile_into(
        &mut self,
        program: Vec<AstNode>,
        bin: &mut Executable,
    ) -> Result<(), CompilationError> {
        for node in program {
            self.compile_statement(bin, &node)?;
        }
        Ok(())
    }

    fn compile_statement(
        &mut self,
        bin: &mut Executable,
        statement_node: &AstNode,
    ) -> Result<(), CompilationError> {
        match statement_node.statement() {
            Statement::Expression { expression } => {
                self.compile_expression(bin, expression)?;
                bin.push_opcode(OpCode::Pop, expression.span);
            }
            Statement::Print { expression, .. } => {
                self.compile_expression(bin, expression)?;
                bin.push_opcode(OpCode::Print, statement_node.span);
            }
            Statement::Declaration {
                name, initializer, ..
            } => {
                let name_value = Value::from(name.clone());

                // Leave the initial value of the variable on the top of the stack
                if let Some(init_expression) = initializer {
                    self.compile_expression(bin, init_expression)?;
                } else {
                    bin.push_constant_inst(OpCode::Constant, Value::Nil, statement_node.span);
                }

                if self.scope_depth == 0 {
                    bin.push_constant_inst(
                        OpCode::DeclareGlobal,
                        name_value.clone(),
                        statement_node.span,
                    );
                    bin.push_constant_inst(OpCode::SetGlobal, name_value, statement_node.span);
                    bin.push_opcode(OpCode::Pop, statement_node.span);
                } else {
                    if let Some(local) = self.resolve_local(name) {
                        if local.0.depth == self.scope_depth {
                            return Err(CompilationError {
                                message: format!("Redeclaration of local variable {}", name),
                                span: statement_node.span,
                            });
                        }
                    }

                    self.locals.push(Local {
                        name: name.clone(),
                        depth: self.scope_depth,
                    });
                }
            }
            Statement::Block {
                declarations,
                rbrace,
            } => {
                self.begin_scope();
                for statement in declarations.iter() {
                    self.compile_statement(bin, statement)?
                }

                self.end_scope(bin, rbrace.span);
            }
            Statement::If {
                condition,
                if_block,
                else_block,
                ..
            } => {
                self.compile_expression(bin, condition)?;
                bin.push_opcode(OpCode::JumpIfFalse, statement_node.span);
                let first_jump = bin.len();
                bin.push_u16(0 as u16, statement_node.span);
                bin.push_opcode(OpCode::Pop, statement_node.span);
                self.compile_statement(bin, if_block)?;

                if bin.len() > u16::max_value() as usize {
                    return Err(CompilationError {
                        message: format!("Binary may not be more than {} bytes long.", bin.len()),
                        span: statement_node.span,
                    });
                }

                bin.push_opcode(OpCode::Jump, statement_node.span);
                let second_jump = bin.len();
                bin.push_u16(0 as u16, statement_node.span);
                bin.replace_u16(first_jump, bin.len() as u16);
                bin.push_opcode(OpCode::Pop, statement_node.span);

                if let Some(else_block) = else_block {
                    self.compile_statement(bin, else_block)?;
                }

                if bin.len() > u16::max_value() as usize {
                    return Err(CompilationError {
                        message: format!("Binary may not be more than {} bytes long.", bin.len()),
                        span: statement_node.span,
                    });
                }
                bin.replace_u16(second_jump, bin.len() as u16);
            }
            Statement::While { condition, block } => {
                let condition_index = bin.len() as u16;
                self.compile_expression(bin, condition)?;
                bin.push_opcode(OpCode::JumpIfFalse, statement_node.span);
                let jump_to_end_index = bin.len();
                bin.push_u16(0, statement_node.span);
                bin.push_opcode(OpCode::Pop, statement_node.span);
                self.compile_statement(bin, block)?;
                bin.push_opcode(OpCode::Jump, statement_node.span);
                bin.push_u16(condition_index, statement_node.span);

                if bin.len() > u16::max_value() as usize {
                    return Err(CompilationError {
                        message: format!("Binary may not be more than {} bytes long.", bin.len()),
                        span: statement_node.span,
                    });
                }
                bin.replace_u16(jump_to_end_index, bin.len() as u16);
                bin.push_opcode(OpCode::Pop, statement_node.span);
            }
            Statement::For {
                initializer,
                condition,
                update,
                block,
            } => {
                self.begin_scope();
                if let Some(initializer) = initializer {
                    self.compile_statement(bin, initializer)?;
                }

                let condition_index = bin.len() as u16;
                let jump_to_end_index = if let Some(condition) = condition {
                    self.compile_expression(bin, condition)?;
                    bin.push_opcode(OpCode::JumpIfFalse, statement_node.span);
                    let jump_to_end_index = bin.len();
                    bin.push_u16(0, statement_node.span);
                    bin.push_opcode(OpCode::Pop, condition.span);
                    jump_to_end_index
                } else {
                    0
                };

                self.compile_statement(bin, block)?;
                if let Some(update) = update {
                    self.compile_expression(bin, update)?;
                    bin.push_opcode(OpCode::Pop, update.span);
                }
                bin.push_opcode(OpCode::Jump, statement_node.span);
                bin.push_u16(condition_index, statement_node.span);

                if condition.is_some() {
                    if bin.len() > u16::max_value() as usize {
                        return Err(CompilationError {
                            message: format!(
                                "Binary may not be more than {} bytes long.",
                                bin.len()
                            ),
                            span: statement_node.span,
                        });
                    }
                    bin.replace_u16(jump_to_end_index, bin.len() as u16);
                }
                bin.push_opcode(OpCode::Pop, statement_node.span);
                self.end_scope(bin, block.span);
            }
            Statement::FunDeclaration {
                name,
                parameters,
                body,
            } => {
                // Empty the list of locals
                let mut locals_backup: Vec<Local> = self.locals.drain(0..).collect();
                self.begin_scope();

                // Add the parameters to the list of Locals
                for param in parameters.iter() {
                    if let Kind::IdentifierLiteral(param_name) = &param.kind {
                        self.locals.push(Local {
                            name: param_name.clone(),
                            depth: self.scope_depth,
                        });
                    } else {
                        return Err(CompilationError {
                            message: "Expected parameter name to be IdentifierLiteral".to_string(),
                            span: param.span,
                        });
                    }
                }

                // Compile the function body
                let mut function_binary = Executable::new(name.clone());
                self.compile_statement(&mut function_binary, body)?;

                // Always add return nil; to the end in case there is no explicit return statement
                function_binary.push_constant_inst(
                    OpCode::Constant,
                    Value::Nil,
                    statement_node.span,
                );
                function_binary.push_opcode(OpCode::Return, body.span);

                if cfg!(feature = "disassemble") {
                    // Disassemble the function body
                    function_binary.dump(self.output_stream);
                }

                // End the scope and restore the outer function's locals
                self.end_scope(&mut function_binary, body.span);
                self.locals = locals_backup.drain(0..).collect();

                // Put the function object on the top of the stack and create a closure
                let value = Value::from(ObjFunction {
                    name: Box::new(Obj::from(name.clone())),
                    arity: parameters.len() as u8,
                    bin: function_binary,
                });
                bin.push_constant_inst(OpCode::Closure, value, statement_node.span);

                // Assign the function to the variable of the matching name
                let name_value = Value::from(name.clone());
                if self.scope_depth == 0 {
                    bin.push_constant_inst(
                        OpCode::DeclareGlobal,
                        name_value.clone(),
                        statement_node.span,
                    );
                    bin.push_constant_inst(OpCode::SetGlobal, name_value, statement_node.span);
                    bin.push_opcode(OpCode::Pop, statement_node.span);
                } else {
                    self.locals.push(Local {
                        name: name.clone(),
                        depth: self.scope_depth,
                    });
                }
            }
            Statement::Return { value } => {
                match value {
                    Some(expression) => {
                        self.compile_expression(bin, expression)?;
                    }
                    None => {
                        bin.push_constant_inst(OpCode::Constant, Value::Nil, statement_node.span);
                    }
                }

                bin.push_opcode(OpCode::Return, statement_node.span)
            }
        };

        Ok(())
    }

    fn compile_expression(
        &mut self,
        bin: &mut Executable,
        expression_node: &AstNode,
    ) -> Result<(), CompilationError> {
        match expression_node.expression() {
            Expression::Constant { value } => {
                bin.push_constant_inst(OpCode::Constant, value.clone(), expression_node.span);
            }
            Expression::Unary {
                operator,
                expression,
            } => {
                self.compile_expression(bin, expression)?;
                match operator.kind {
                    Kind::Minus => bin.push_opcode(OpCode::Negate, expression_node.span),
                    Kind::Bang => bin.push_opcode(OpCode::Not, expression_node.span),
                    _ => {
                        return Err(CompilationError {
                            message: format!("Invalid unary operator {:?}", operator),
                            span: operator.span,
                        })
                    }
                }
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                self.compile_expression(bin, left)?;
                self.compile_expression(bin, right)?;
                match operator.kind {
                    Kind::Plus => bin.push_opcode(OpCode::Add, expression_node.span),
                    Kind::Minus => bin.push_opcode(OpCode::Subtract, expression_node.span),
                    Kind::Star => bin.push_opcode(OpCode::Multiply, expression_node.span),
                    Kind::Slash => bin.push_opcode(OpCode::Divide, expression_node.span),
                    Kind::Less => bin.push_opcode(OpCode::Less, expression_node.span),
                    Kind::LessEqual => bin.push_opcode(OpCode::LessEqual, expression_node.span),
                    Kind::Greater => bin.push_opcode(OpCode::Greater, expression_node.span),
                    Kind::GreaterEqual => {
                        bin.push_opcode(OpCode::GreaterEqual, expression_node.span)
                    }
                    Kind::EqualEqual => bin.push_opcode(OpCode::Equal, expression_node.span),
                    Kind::BangEqual => {
                        bin.push_opcode(OpCode::Equal, expression_node.span);
                        bin.push_opcode(OpCode::Not, expression_node.span);
                    }
                    _ => {
                        return Err(CompilationError {
                            message: format!("Invalid binary operator {:?}", operator),
                            span: operator.span,
                        })
                    }
                }
            }
            Expression::Assignment { lvalue, rvalue, .. } => {
                if let Expression::Variable { name } = lvalue.expression() {
                    self.compile_expression(bin, rvalue)?;
                    match self.resolve_local(name) {
                        Some((_, index)) => {
                            bin.push_opcode(OpCode::SetLocal, lvalue.span);
                            bin.push_u8(index as u8, lvalue.span);
                        }
                        None => {
                            let name_value = Value::from(name.clone());
                            bin.push_constant_inst(
                                OpCode::SetGlobal,
                                name_value,
                                expression_node.span,
                            );
                        }
                    }
                } else {
                    return Err(CompilationError {
                        message: format!("Assignment to non-lvalue {:?}", lvalue),
                        span: lvalue.span,
                    });
                }
            }
            Expression::Variable { name } => match self.resolve_local(name) {
                Some((_, index)) => {
                    bin.push_opcode(OpCode::GetLocal, expression_node.span);
                    bin.push_u8(index as u8, expression_node.span);
                }
                None => {
                    let name_value = Value::from(name.clone());
                    bin.push_constant_inst(OpCode::GetGlobal, name_value, expression_node.span);
                }
            },
            Expression::Call { target, arguments } => {
                self.compile_expression(bin, target)?;
                for arg in arguments {
                    self.compile_expression(bin, arg)?;
                }
                bin.push_opcode(OpCode::Call, expression_node.span);
                bin.push_u8(arguments.len() as u8, expression_node.span);
            }
        };

        Ok(())
    }

    fn resolve_local(&self, name: &str) -> Option<(&Local, usize)> {
        for (index, local) in self.locals.iter().rev().enumerate() {
            if local.name == name {
                return Some((&local, self.locals.len() - index - 1));
            }
        }
        None
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self, bin: &mut Executable, end_span: Span) {
        while let Some(local) = self.locals.last() {
            if local.depth == self.scope_depth {
                bin.push_opcode(OpCode::Pop, end_span);
                self.locals.pop();
            } else {
                break;
            }
        }
        self.scope_depth -= 1;
    }
}
