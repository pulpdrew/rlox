use crate::ast::{AstNode, SpannedAstNode};
use crate::error::ReportableError;
use crate::executable::Executable;
use crate::object::{ObjClass, ObjClosure, ObjFunction, ObjString};
use crate::opcode::OpCode;
use crate::token::{Kind, Span};
use crate::value::Value;
use std::collections::VecDeque;
use std::io::Write;
use std::rc::Rc;

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
    frames: VecDeque<Frame>,
    output_stream: &'a mut W,
}

pub fn compile<W: Write>(
    program: Vec<SpannedAstNode>,
    output_stream: &mut W,
) -> Result<ObjClosure, CompilationError> {
    let mut compiler = Compiler::new(output_stream);
    let mut bin = Executable::new(String::from("script"));

    for node in program {
        compiler.compile_node(&mut bin, &node)?;
    }

    Ok(ObjClosure {
        function: Rc::new(ObjFunction {
            arity: 0,
            bin,
            name: Box::new(ObjString::from("script")),
            upvalues: vec![],
        }),
        upvalues: vec![],
    })
}

impl<'a, W: Write> Compiler<'a, W> {
    pub fn new(output_stream: &'a mut W) -> Self {
        let mut scopes = VecDeque::new();
        scopes.push_back(Frame::new(true));
        Compiler {
            frames: scopes,
            output_stream,
        }
    }

    fn compile_node(
        &mut self,
        bin: &mut Executable,
        spanned_node: &SpannedAstNode,
    ) -> Result<(), CompilationError> {
        let (node, node_span) = destructure_node(spanned_node)?;

        match node {
            AstNode::ExpressionStmt { expression } => {
                self.compile_node(bin, expression)?;
                bin.push_opcode(OpCode::Pop, expression.span);
            }
            AstNode::Print { expression, .. } => {
                self.compile_node(bin, expression)?;
                bin.push_opcode(OpCode::Print, node_span);
            }
            AstNode::VarDeclaration {
                name, initializer, ..
            } => {
                let name_value = Value::from(name.clone());

                // Leave the initial value of the variable on the top of the stack
                if let Some(init_expression) = initializer {
                    self.compile_node(bin, init_expression)?;
                } else {
                    bin.push_constant_inst(OpCode::Constant, Value::Nil, node_span);
                }

                if self.current_frame().is_global() {
                    bin.push_constant_inst(OpCode::DeclareGlobal, name_value.clone(), node_span);
                    bin.push_constant_inst(OpCode::SetGlobal, name_value, node_span);
                    bin.push_opcode(OpCode::Pop, node_span);
                } else {
                    if let Some((_, distance)) = self.current_frame().resolve_local(name) {
                        if distance == 0 {
                            return Err(CompilationError {
                                message: format!("Redeclaration of local variable {}", name),
                                span: node_span,
                            });
                        }
                    }

                    self.current_frame_mut().add_local(name);
                }
            }
            AstNode::ClassDeclaration { name, .. } => {
                let name_value = Value::from(name.clone());
                let class = Value::from(ObjClass {
                    name: Box::new(ObjString::from(name.clone())),
                });
                bin.push_constant_inst(OpCode::Constant, class, node_span);

                if self.current_frame().is_global() {
                    bin.push_constant_inst(OpCode::DeclareGlobal, name_value.clone(), node_span);
                    bin.push_constant_inst(OpCode::SetGlobal, name_value, node_span);
                    bin.push_opcode(OpCode::Pop, node_span);
                } else {
                    if let Some((_, distance)) = self.current_frame().resolve_local(name) {
                        if distance == 0 {
                            return Err(CompilationError {
                                message: format!("Redeclaration of local variable {}", name),
                                span: node_span,
                            });
                        }
                    }

                    self.current_frame_mut().add_local(name);
                }
            }
            AstNode::Block {
                declarations,
                rbrace,
            } => {
                self.current_frame_mut().begin_scope();
                for statement in declarations.iter() {
                    self.compile_node(bin, statement)?
                }

                self.current_frame_mut().end_scope(bin, rbrace.span);
            }
            AstNode::If {
                condition,
                if_block,
                else_block,
                ..
            } => {
                self.compile_node(bin, condition)?;
                bin.push_opcode(OpCode::JumpIfFalse, node_span);
                let first_jump = bin.len();
                bin.push_u16(0 as u16, node_span);
                bin.push_opcode(OpCode::Pop, node_span);
                self.compile_node(bin, if_block)?;

                if bin.len() > u16::max_value() as usize {
                    return Err(CompilationError {
                        message: format!("Binary may not be more than {} bytes long.", bin.len()),
                        span: node_span,
                    });
                }

                bin.push_opcode(OpCode::Jump, node_span);
                let second_jump = bin.len();
                bin.push_u16(0 as u16, node_span);
                bin.replace_u16(first_jump, bin.len() as u16);
                bin.push_opcode(OpCode::Pop, node_span);

                if let Some(else_block) = else_block {
                    self.compile_node(bin, else_block)?;
                }

                if bin.len() > u16::max_value() as usize {
                    return Err(CompilationError {
                        message: format!("Binary may not be more than {} bytes long.", bin.len()),
                        span: node_span,
                    });
                }
                bin.replace_u16(second_jump, bin.len() as u16);
            }
            AstNode::While { condition, block } => {
                let condition_index = bin.len() as u16;
                self.compile_node(bin, condition)?;
                bin.push_opcode(OpCode::JumpIfFalse, node_span);
                let jump_to_end_index = bin.len();
                bin.push_u16(0, node_span);
                bin.push_opcode(OpCode::Pop, node_span);
                self.compile_node(bin, block)?;
                bin.push_opcode(OpCode::Jump, node_span);
                bin.push_u16(condition_index, node_span);

                if bin.len() > u16::max_value() as usize {
                    return Err(CompilationError {
                        message: format!("Binary may not be more than {} bytes long.", bin.len()),
                        span: node_span,
                    });
                }
                bin.replace_u16(jump_to_end_index, bin.len() as u16);
                bin.push_opcode(OpCode::Pop, node_span);
            }
            AstNode::For {
                initializer,
                condition,
                update,
                block,
            } => {
                self.current_frame_mut().begin_scope();
                if let Some(initializer) = initializer {
                    self.compile_node(bin, initializer)?;
                }

                let condition_index = bin.len() as u16;
                let jump_to_end_index = if let Some(condition) = condition {
                    self.compile_node(bin, condition)?;
                    bin.push_opcode(OpCode::JumpIfFalse, node_span);
                    let jump_to_end_index = bin.len();
                    bin.push_u16(0, node_span);
                    bin.push_opcode(OpCode::Pop, condition.span);
                    jump_to_end_index
                } else {
                    0
                };

                self.compile_node(bin, block)?;
                if let Some(update) = update {
                    self.compile_node(bin, update)?;
                    bin.push_opcode(OpCode::Pop, update.span);
                }
                bin.push_opcode(OpCode::Jump, node_span);
                bin.push_u16(condition_index, node_span);

                if condition.is_some() {
                    if bin.len() > u16::max_value() as usize {
                        return Err(CompilationError {
                            message: format!(
                                "Binary may not be more than {} bytes long.",
                                bin.len()
                            ),
                            span: node_span,
                        });
                    }
                    bin.replace_u16(jump_to_end_index, bin.len() as u16);
                }
                bin.push_opcode(OpCode::Pop, node_span);
                self.current_frame_mut().end_scope(bin, block.span);
            }
            AstNode::FunDeclaration {
                name,
                parameters,
                body,
            } => {
                self.frames.push_back(Frame::new(false));

                // Add the parameters to the list of Locals
                for param in parameters.iter() {
                    if let Kind::IdentifierLiteral(param_name) = &param.kind {
                        self.current_frame_mut().add_local(param_name);
                    } else {
                        return Err(CompilationError {
                            message: "Expected parameter name to be IdentifierLiteral".to_string(),
                            span: param.span,
                        });
                    }
                }

                // Compile the function body
                let mut function_binary = Executable::new(name.clone());
                self.compile_node(&mut function_binary, body)?;

                // Always add return nil; to the end in case there is no explicit return statement
                function_binary.push_constant_inst(OpCode::Constant, Value::Nil, node_span);
                function_binary.push_opcode(OpCode::Return, body.span);

                if cfg!(feature = "disassemble") {
                    // Disassemble the function body
                    function_binary.dump(self.output_stream);
                }

                // End the scope and restore the outer function's locals
                self.current_frame_mut()
                    .end_scope(&mut function_binary, body.span);
                self.frames.pop_back();

                // Put the function object on the top of the stack and create a closure
                let value = Value::from(ObjFunction {
                    name: Box::new(ObjString::from(name.clone())),
                    arity: parameters.len() as u8,
                    bin: function_binary,
                    upvalues: self.current_frame_mut().upvalues.drain(0..).collect(),
                });
                bin.push_constant_inst(OpCode::Closure, value, node_span);

                // Assign the function to the variable of the matching name
                let name_value = Value::from(name.clone());
                if self.current_frame().is_global() {
                    bin.push_constant_inst(OpCode::DeclareGlobal, name_value.clone(), node_span);
                    bin.push_constant_inst(OpCode::SetGlobal, name_value, node_span);
                    bin.push_opcode(OpCode::Pop, node_span);
                } else {
                    self.current_frame_mut().add_local(name);
                }
            }
            AstNode::Return { value } => {
                match value {
                    Some(expression) => {
                        self.compile_node(bin, expression)?;
                    }
                    None => {
                        bin.push_constant_inst(OpCode::Constant, Value::Nil, node_span);
                    }
                }

                bin.push_opcode(OpCode::Return, node_span)
            }
            AstNode::Constant { value } => {
                bin.push_constant_inst(OpCode::Constant, value.clone(), node_span);
            }
            AstNode::Unary {
                operator,
                expression,
            } => {
                self.compile_node(bin, expression)?;
                match operator.kind {
                    Kind::Minus => bin.push_opcode(OpCode::Negate, node_span),
                    Kind::Bang => bin.push_opcode(OpCode::Not, node_span),
                    _ => {
                        return Err(CompilationError {
                            message: format!("Invalid unary operator {:?}", operator),
                            span: operator.span,
                        })
                    }
                }
            }
            AstNode::Binary {
                left,
                operator,
                right,
            } => {
                self.compile_node(bin, left)?;
                self.compile_node(bin, right)?;
                match operator.kind {
                    Kind::Plus => bin.push_opcode(OpCode::Add, node_span),
                    Kind::Minus => bin.push_opcode(OpCode::Subtract, node_span),
                    Kind::Star => bin.push_opcode(OpCode::Multiply, node_span),
                    Kind::Slash => bin.push_opcode(OpCode::Divide, node_span),
                    Kind::Less => bin.push_opcode(OpCode::Less, node_span),
                    Kind::LessEqual => bin.push_opcode(OpCode::LessEqual, node_span),
                    Kind::Greater => bin.push_opcode(OpCode::Greater, node_span),
                    Kind::GreaterEqual => bin.push_opcode(OpCode::GreaterEqual, node_span),
                    Kind::EqualEqual => bin.push_opcode(OpCode::Equal, node_span),
                    Kind::BangEqual => {
                        bin.push_opcode(OpCode::Equal, node_span);
                        bin.push_opcode(OpCode::Not, node_span);
                    }
                    _ => {
                        return Err(CompilationError {
                            message: format!("Invalid binary operator {:?}", operator),
                            span: operator.span,
                        })
                    }
                }
            }
            AstNode::Assignment { lvalue, rvalue, .. } => {
                if let Some(AstNode::Variable { name }) = &lvalue.node {
                    self.compile_node(bin, rvalue)?;
                    if let Some((index, _)) = self.current_frame().resolve_local(name) {
                        bin.push_opcode(OpCode::SetLocal, node_span);
                        bin.push_u8(index as u8, node_span);
                    } else if let Some(index) = self.resolve_upvalue(0, name) {
                        bin.push_opcode(OpCode::SetUpvalue, node_span);
                        bin.push_u8(index as u8, node_span);
                    } else {
                        let name_value = Value::from(name.clone());
                        bin.push_constant_inst(OpCode::SetGlobal, name_value, node_span);
                    }
                } else {
                    return Err(CompilationError {
                        message: format!("Assignment to non-lvalue {:?}", lvalue),
                        span: lvalue.span,
                    });
                }
            }
            AstNode::Variable { name } => {
                if let Some((index, _)) = self.current_frame().resolve_local(name) {
                    bin.push_opcode(OpCode::GetLocal, node_span);
                    bin.push_u8(index as u8, node_span);
                } else if let Some(index) = self.resolve_upvalue(0, name) {
                    bin.push_opcode(OpCode::GetUpvalue, node_span);
                    bin.push_u8(index as u8, node_span);
                } else {
                    let name_value = Value::from(name.clone());
                    bin.push_constant_inst(OpCode::GetGlobal, name_value, node_span);
                }
            }
            AstNode::Call { target, arguments } => {
                self.compile_node(bin, target)?;
                for arg in arguments {
                    self.compile_node(bin, arg)?;
                }
                bin.push_opcode(OpCode::Call, node_span);
                bin.push_u8(arguments.len() as u8, node_span);
            }
        };

        Ok(())
    }

    fn current_frame(&self) -> &Frame {
        self.frames.back().unwrap()
    }

    fn current_frame_mut(&mut self) -> &mut Frame {
        self.frames.back_mut().unwrap()
    }

    fn resolve_upvalue(&mut self, frame_depth: usize, name: &str) -> Option<usize> {
        if frame_depth >= self.frames.len() {
            return None;
        }

        if let Some((index, _)) = self
            .frames
            .get_mut(self.frames.len() - frame_depth - 1)
            .unwrap()
            .resolve_local(name)
        {
            return Some(
                self.frames
                    .get_mut(self.frames.len() - frame_depth - 1)
                    .unwrap()
                    .add_upvalue(index, true),
            );
        }

        if let Some(index) = self.resolve_upvalue(frame_depth + 1, name) {
            Some(
                self.frames
                    .get_mut(self.frames.len() - frame_depth - 1)
                    .unwrap()
                    .add_upvalue(index, false),
            )
        } else {
            None
        }
    }
}

fn destructure_node(node: &SpannedAstNode) -> Result<(&AstNode, Span), CompilationError> {
    if let SpannedAstNode {
        node: Some(node),
        span,
    } = node
    {
        Ok((node, *span))
    } else {
        Err(CompilationError {
            message: "Attempted to compile SpannedAstNode with node: None".to_string(),
            span: node.span,
        })
    }
}

/// A record of all the variables declared in a single scope
#[derive(Debug)]
struct LocalScope {
    pub offset: usize,
    locals: Vec<String>,
}

impl LocalScope {
    fn new(offset: usize) -> Self {
        LocalScope {
            locals: vec![],
            offset,
        }
    }

    fn resolve(&self, name: &str) -> Option<usize> {
        for (index, n) in self.locals.iter().enumerate() {
            if name == n {
                return Some(index);
            }
        }
        None
    }

    fn push(&mut self, name: String) {
        self.locals.push(name);
    }

    fn len(&self) -> usize {
        self.locals.len()
    }
}

/// A record of all the variables declared in a single function
#[derive(Debug)]
struct Frame {
    scopes: VecDeque<LocalScope>,
    upvalues: VecDeque<(bool, usize)>,
    is_global: bool,
}

impl Frame {
    fn new(is_global: bool) -> Self {
        let mut scopes = VecDeque::new();
        scopes.push_back(LocalScope::new(0));
        Frame {
            scopes,
            is_global,
            upvalues: VecDeque::new(),
        }
    }

    fn add_local(&mut self, name: &str) {
        self.scopes.back_mut().unwrap().push(name.to_string());
    }

    fn add_upvalue(&mut self, index: usize, is_local: bool) -> usize {
        for (i, upvalue) in self.upvalues.iter().enumerate() {
            if upvalue.0 == is_local && upvalue.1 == index {
                return i;
            }
        }

        self.upvalues.push_back((is_local, index));
        self.upvalues.len() - 1
    }

    /// Resolves a local to (offset from frame pointer, distance to scope)
    fn resolve_local(&self, name: &str) -> Option<(usize, usize)> {
        for (distance, scope) in self.scopes.iter().rev().enumerate() {
            if let Some(offset) = scope.resolve(name) {
                return Some((offset + scope.offset, distance));
            }
        }
        None
    }

    fn is_global(&self) -> bool {
        self.is_global && self.scopes.len() == 1
    }

    fn begin_scope(&mut self) {
        let new_scope = match self.scopes.back() {
            Some(parent) => LocalScope::new(parent.offset + parent.len()),
            None => LocalScope::new(0),
        };
        self.scopes.push_back(new_scope)
    }

    fn end_scope(&mut self, bin: &mut Executable, end_span: Span) {
        for _ in 0..self.scopes.back().unwrap().len() {
            bin.push_opcode(OpCode::Pop, end_span);
        }
        self.scopes.pop_back();
    }
}
