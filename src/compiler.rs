use crate::ast::{AstNode, SpannedAstNode};
use crate::error::CompilerError;
use crate::executable::Executable;
use crate::object::{ObjClass, ObjClosure, ObjFunction, ObjString};
use crate::opcode::OpCode;
use crate::token::{Kind, Span};
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::Write;
use std::rc::Rc;

/// The state of a compiler
#[derive(Debug)]
pub struct Compiler<'a, W: Write> {
    /// The call frames (containing variables) that are expected
    /// to be on the stack when the code currently being compiled
    /// is executed.
    frames: VecDeque<Frame>,

    /// The Write stream that compilation output is written to
    output_stream: &'a mut W,
}

/// Compile the given AST root nodes into an executable
///
/// Returns a closure representing the executable script if compilation is successful.
/// Returns a `CompilerError` if compilation is unsuccessful.
///
/// # Arguments
///
/// * `program` - the declaration nodes that make up the program to be compiled
/// * `output_stream` - the Write stream that any compilation output should be written to
pub fn compile<W: Write>(
    program: Vec<SpannedAstNode>,
    output_stream: &mut W,
) -> Result<ObjClosure, CompilerError> {
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
        upvalues: RefCell::new(vec![]),
    })
}

impl<'a, W: Write> Compiler<'a, W> {
    /// A new compiler with only a global scope defined.
    pub fn new(output_stream: &'a mut W) -> Self {
        let mut scopes = VecDeque::new();
        scopes.push_back(Frame::new(true, FunctionType::None));
        Compiler {
            frames: scopes,
            output_stream,
        }
    }

    /// Compile a single AST node into the provided binary.
    fn compile_node(
        &mut self,
        bin: &mut Executable,
        spanned_node: &SpannedAstNode,
    ) -> Result<(), CompilerError> {
        let (node, node_span) = spanned_node.destructure()?;

        match node {
            AstNode::Unary {
                operator,
                expression,
            } => {
                self.compile_node(bin, expression)?;
                match operator.kind {
                    Kind::Minus => {
                        bin.push_opcode(OpCode::Negate, node_span);
                    }
                    Kind::Bang => {
                        bin.push_opcode(OpCode::Not, node_span);
                    }
                    _ => {
                        return Err(CompilerError {
                            message: format!("Invalid unary operator '{}'", operator.kind),
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

                let opcode = match operator.kind {
                    Kind::Plus => OpCode::Add,
                    Kind::Minus => OpCode::Subtract,
                    Kind::Star => OpCode::Multiply,
                    Kind::Slash => OpCode::Divide,
                    Kind::Less => OpCode::Less,
                    Kind::LessEqual => OpCode::LessEqual,
                    Kind::Greater => OpCode::Greater,
                    Kind::GreaterEqual => OpCode::GreaterEqual,
                    Kind::EqualEqual => OpCode::Equal,
                    Kind::BangEqual => OpCode::NotEqual,
                    _ => {
                        return Err(CompilerError {
                            message: format!("Invalid binary operator '{}'", operator.kind),
                            span: operator.span,
                        });
                    }
                };
                bin.push_opcode(opcode, node_span);
            }
            AstNode::Assignment { lvalue, rvalue, .. } => match &lvalue.node {
                Some(AstNode::Variable { name }) => {
                    self.compile_node(bin, rvalue)?;
                    self.set_variable(name, bin, &node_span);
                }
                Some(AstNode::FieldAccess { target, name }) => {
                    self.compile_node(bin, target)?;
                    self.compile_node(bin, rvalue)?;
                    let index = bin.add_constant(Value::from(name.to_string()));
                    bin.push_opcode(OpCode::SetField(index), node_span);
                }
                _ => {
                    return Err(CompilerError {
                        message: format!("Assignment to non-lvalue {:?}", lvalue),
                        span: lvalue.span,
                    });
                }
            },
            AstNode::Variable { name } => {
                if name == &"this".to_string() && !self.currently_within_method() {
                    return Err(CompilerError {
                        message: "Cannot use 'this' outside of a class method.".to_string(),
                        span: node_span,
                    });
                }
                self.get_variable(name, bin, &node_span);
            }
            AstNode::Constant { value } => {
                let index = bin.add_constant(value.clone());
                bin.push_opcode(OpCode::Constant(index), node_span);
            }
            AstNode::Invokation { target, arguments } => {
                self.compile_node(bin, target)?;

                // Empty stack slot to be replaced by `this` when the target is a method
                let index = bin.add_constant(Value::Nil);
                bin.push_opcode(OpCode::Constant(index), node_span);

                for arg in arguments {
                    self.compile_node(bin, arg)?;
                }
                bin.push_opcode(OpCode::Invoke(arguments.len()), node_span);
            }
            AstNode::FieldAccess { target, name } => {
                self.compile_node(bin, target)?;
                let index = bin.add_constant(Value::from(name.to_string()));
                bin.push_opcode(OpCode::ReadField(index), node_span);
            }
            AstNode::SuperAccess { name } => {
                // Put the current instance on the stack
                if let Some((index, _)) = self.current_frame().resolve_local("this") {
                    bin.push_opcode(OpCode::GetLocal(index), node_span);
                } else {
                    return Err(CompilerError {
                        message: "'super' may not be used outside methods".to_string(),
                        span: node_span,
                    });
                }

                // Put the superclass on the stack
                if let Some(index) = self.resolve_upvalue(0, "super") {
                    bin.push_opcode(OpCode::GetUpvalue(index), node_span);
                } else {
                    return Err(CompilerError {
                        message: "No superclass available here".to_string(),
                        span: node_span,
                    });
                }

                let index = bin.add_constant(Value::from(name.to_string()));
                bin.push_opcode(OpCode::GetSuper(index), node_span);
            }
            AstNode::ClassDeclaration {
                name,
                methods,
                superclass,
            } => {
                // Create an empty class and bind it to a variable
                let class = Value::from(ObjClass {
                    name: Box::new(ObjString::from(name.clone())),
                    methods: RefCell::new(HashMap::new()),
                });
                let index = bin.add_constant(class);
                bin.push_opcode(OpCode::Constant(index), node_span);
                self.declare_variable(name, bin, &node_span)?;

                // Leave the superclass on the stack to be captured by any super calls
                if let Some(superclass_name) = superclass {
                    self.current_frame_mut().begin_scope();
                    self.get_variable(superclass_name, bin, &node_span);
                    self.declare_variable("super", bin, &node_span)?;
                }

                // Put the new class on the top of the stack
                self.get_variable(name, bin, &node_span);

                // Inherit from the superclass if there is one
                if superclass.is_some() {
                    bin.push_opcode(OpCode::Inherit, node_span);
                }

                // Compile each method and add to the class
                for SpannedAstNode { node, span } in methods {
                    self.function_declaration(
                        bin,
                        &node.as_ref().unwrap(),
                        node_span,
                        FunctionType::Method,
                    )?;
                    bin.push_opcode(OpCode::Method, *span);
                }

                // Pop the class, then the superclass
                bin.push_opcode(OpCode::Pop, node_span);
                if superclass.is_some() {
                    self.current_frame_mut().end_scope(bin, node_span);
                }
            }
            AstNode::FunDeclaration { name, .. } => {
                self.function_declaration(bin, node, node_span, FunctionType::Function)?;
                self.declare_variable(name, bin, &node_span)?;
            }
            AstNode::VarDeclaration {
                name, initializer, ..
            } => {
                // Leave the initial value of the variable on the top of the stack
                if let Some(init_expression) = initializer {
                    self.compile_node(bin, init_expression)?;
                } else {
                    let index = bin.add_constant(Value::Nil);
                    bin.push_opcode(OpCode::Constant(index), node_span);
                }
                self.declare_variable(name, bin, &node_span)?;
            }
            AstNode::ExpressionStmt { expression } => {
                self.compile_node(bin, expression)?;
                bin.push_opcode(OpCode::Pop, expression.span);
            }
            AstNode::Print { expression, .. } => {
                self.compile_node(bin, expression)?;
                bin.push_opcode(OpCode::Print, node_span);
            }
            AstNode::Return { value } => {
                match value {
                    Some(expression) => {
                        self.compile_node(bin, expression)?;
                    }
                    None => {
                        let index = bin.add_constant(Value::Nil);
                        bin.push_opcode(OpCode::Constant(index), node_span);
                    }
                }
                bin.push_opcode(OpCode::Return, node_span);
            }
            AstNode::Block { declarations } => {
                self.current_frame_mut().begin_scope();
                for statement in declarations.iter() {
                    self.compile_node(bin, statement)?
                }
                self.current_frame_mut().end_scope(bin, node_span);
            }
            AstNode::If {
                condition,
                if_block,
                else_block,
                ..
            } => {
                self.compile_node(bin, condition)?;
                let first_jump = bin.push_opcode(OpCode::JumpIfFalse(0), node_span);
                bin.push_opcode(OpCode::Pop, node_span);
                self.compile_node(bin, if_block)?;

                bin.assert_not_too_long(&node_span)?;

                let second_jump = bin.push_opcode(OpCode::Jump(0), node_span);
                bin[first_jump] = OpCode::JumpIfFalse(bin.len());
                bin.push_opcode(OpCode::Pop, node_span);

                if let Some(else_block) = else_block {
                    self.compile_node(bin, else_block)?;
                }

                bin.assert_not_too_long(&node_span)?;
                bin[second_jump] = OpCode::Jump(bin.len());
            }
            AstNode::While { condition, block } => {
                let condition_index = bin.len();
                self.compile_node(bin, condition)?;
                let jump_to_end_index = bin.push_opcode(OpCode::JumpIfFalse(0), node_span);
                bin.push_opcode(OpCode::Pop, node_span);
                self.compile_node(bin, block)?;
                bin.push_opcode(OpCode::Jump(condition_index), node_span);

                bin.assert_not_too_long(&node_span)?;
                bin[jump_to_end_index] = OpCode::JumpIfFalse(bin.len());
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

                let condition_index = bin.len();
                let jump_to_end_index = if let Some(condition) = condition {
                    self.compile_node(bin, condition)?;
                    let jump_to_end_index = bin.push_opcode(OpCode::JumpIfFalse(0), node_span);
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
                bin.push_opcode(OpCode::Jump(condition_index), node_span);

                if condition.is_some() {
                    bin.assert_not_too_long(&node_span)?;
                    bin[jump_to_end_index] = OpCode::JumpIfFalse(bin.len())
                }
                bin.push_opcode(OpCode::Pop, node_span);
                self.current_frame_mut().end_scope(bin, block.span);
            }
            AstNode::Or { left, right } => {
                self.compile_node(bin, left)?;
                let jump_index = bin.push_opcode(OpCode::JumpIfTrue(0), node_span);
                bin.push_opcode(OpCode::Pop, node_span);
                self.compile_node(bin, right)?;
                bin[jump_index] = OpCode::JumpIfTrue(bin.len());
                bin.push_opcode(OpCode::Bool, node_span);
            }
            AstNode::And { left, right } => {
                self.compile_node(bin, left)?;
                let jump_index = bin.push_opcode(OpCode::JumpIfFalse(0), node_span);
                bin.push_opcode(OpCode::Pop, node_span);
                self.compile_node(bin, right)?;
                bin[jump_index] = OpCode::JumpIfFalse(bin.len());
                bin.push_opcode(OpCode::Bool, node_span);
            }
        };

        Ok(())
    }

    /// Emit the instructions to bind a new variable to the value that
    /// is at the top of the stack. Consumes the value at the top of the
    /// stack.
    fn declare_variable(
        &mut self,
        name: &str,
        bin: &mut Executable,
        span: &Span,
    ) -> Result<(), CompilerError> {
        let name_value = Value::from(name);

        if self.current_frame().is_global() {
            let index = bin.add_constant(name_value.clone());
            bin.push_opcode(OpCode::DeclareGlobal(index), *span);
            let index = bin.add_constant(name_value);
            bin.push_opcode(OpCode::SetGlobal(index), *span);
            bin.push_opcode(OpCode::Pop, *span);
        } else {
            if let Some((_, distance)) = self.current_frame().resolve_local(name) {
                if distance == 0 {
                    return Err(CompilerError {
                        message: format!("Redeclaration of local variable {}", name),
                        span: *span,
                    });
                }
            }
            self.current_frame_mut().add_local(name);
        }

        Ok(())
    }

    /// Emit the instructions to set an existing variable to the value at the top of the stack.
    /// Does not consume the value at the top of the stack.
    fn set_variable(&mut self, name: &str, bin: &mut Executable, span: &Span) {
        if let Some((index, _)) = self.current_frame().resolve_local(name) {
            bin.push_opcode(OpCode::SetLocal(index), *span);
        } else if let Some(index) = self.resolve_upvalue(0, name) {
            bin.push_opcode(OpCode::SetUpvalue(index), *span);
        } else {
            let name_value = Value::from(name);
            let index = bin.add_constant(name_value);
            bin.push_opcode(OpCode::SetGlobal(index), *span);
        }
    }

    /// Emit the instructions to load a variable onto the top of the stack.
    /// Prioritize local variables over upvalues (closure variables) over
    /// global variables.
    fn get_variable(&mut self, name: &str, bin: &mut Executable, span: &Span) {
        if let Some((index, _)) = self.current_frame().resolve_local(name) {
            bin.push_opcode(OpCode::GetLocal(index), *span);
        } else if let Some(index) = self.resolve_upvalue(0, name) {
            bin.push_opcode(OpCode::GetUpvalue(index), *span);
        } else {
            let name_value = Value::from(name);
            let index = bin.add_constant(name_value);
            bin.push_opcode(OpCode::GetGlobal(index), *span);
        }
    }

    /// Get a reference to the current stack frame
    fn current_frame(&self) -> &Frame {
        self.frames.back().unwrap()
    }

    /// Get a mutable reference to the current stack frame
    fn current_frame_mut(&mut self) -> &mut Frame {
        self.frames.back_mut().unwrap()
    }

    /// Indicates whether or not there is some frame on the stack that
    /// belongs to a method, indicating the validity of `this`
    fn currently_within_method(&self) -> bool {
        self.frames
            .iter()
            .any(|f| f.function_type == FunctionType::Method)
    }

    /// Resolves a variable name to an upvalue index, if possible.
    ///
    /// First looks for an existing upvalue. If not found, then creates a new
    /// one, if it can find the referenced variable on the stack.
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

    /// Compiles a function or method definition and leaves a closure
    /// containing the function on the top of the stack
    fn function_declaration(
        &mut self,
        bin: &mut Executable,
        function_node: &AstNode,
        function_span: Span,
        function_type: FunctionType,
    ) -> Result<(), CompilerError> {
        if let AstNode::FunDeclaration {
            name,
            parameters,
            body,
        } = function_node
        {
            // Track the frame that will be on the call stack at runtime
            let mut function_frame = Frame::new(false, function_type);

            // Add "this" as a local for methods, or a dummy parameter for functions
            if function_type == FunctionType::Method {
                function_frame.add_local("this");
            } else {
                function_frame.add_local("");
            }

            // Add the parameters to the list of Locals
            for param in parameters.iter() {
                if let Kind::IdentifierLiteral(param_name) = &param.kind {
                    function_frame.add_local(param_name);
                } else {
                    return Err(CompilerError {
                        message: "Expected parameter name to be IdentifierLiteral".to_string(),
                        span: param.span,
                    });
                }
            }

            // Push the frame so that nested functions can see it
            self.frames.push_back(function_frame);

            // Compile the function body
            let mut function_binary = Executable::new(name.clone());
            self.compile_node(&mut function_binary, body)?;

            // Always add return nil; to the end in case there is no explicit return statement
            let index = function_binary.add_constant(Value::Nil);
            function_binary.push_opcode(OpCode::Constant(index), function_span);
            function_binary.push_opcode(OpCode::Return, body.span);

            // Disassemble the function body if enabled
            if cfg!(feature = "disassemble") {
                function_binary.dump(self.output_stream);
            }

            // End the scope and restore the outer function's frame
            self.current_frame_mut()
                .end_scope(&mut function_binary, body.span);
            self.frames.pop_back();

            // Put the function object on the top of the stack and create a closure
            let function_value = Value::from(ObjFunction {
                name: Box::new(ObjString::from(name.clone())),
                arity: parameters.len() as u8,
                bin: function_binary,
                upvalues: self.current_frame_mut().upvalues.drain(0..).collect(),
            });
            let index = bin.add_constant(function_value);
            bin.push_opcode(OpCode::Closure(index), function_span);

            Ok(())
        } else {
            Err(CompilerError {
                message: "compiler.function_declaration called with non-FunctionDeclaration node"
                    .to_string(),
                span: function_span,
            })
        }
    }
}

impl Executable {
    /// Errors if self is longer than the executable length limit
    fn assert_not_too_long(&self, span: &Span) -> Result<(), CompilerError> {
        if self.len() > u16::max_value() as usize {
            Err(CompilerError {
                message: format!(
                    "Binary may not be more than {} bytes long.",
                    u16::max_value()
                ),
                span: *span,
            })
        } else {
            Ok(())
        }
    }
}

impl SpannedAstNode {
    /// Converts a `&SpannedAstNode` to a `(&AstNode, Span)` tuple, erroring if
    /// the AstNode is None.
    fn destructure(&self) -> Result<(&AstNode, Span), CompilerError> {
        if let SpannedAstNode {
            node: Some(node),
            span,
        } = self
        {
            Ok((node, *span))
        } else {
            Err(CompilerError {
                message: "Attempted to compile SpannedAstNode with node: None".to_string(),
                span: self.span,
            })
        }
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FunctionType {
    None,
    Function,
    Method,
}

/// A record of all the variables declared in a single function
#[derive(Debug)]
struct Frame {
    scopes: VecDeque<LocalScope>,
    upvalues: VecDeque<(bool, usize)>,
    is_global: bool,
    function_type: FunctionType,
}

impl Frame {
    fn new(is_global: bool, function_type: FunctionType) -> Self {
        let mut scopes = VecDeque::new();
        scopes.push_back(LocalScope::new(0));
        Frame {
            scopes,
            is_global,
            upvalues: VecDeque::new(),
            function_type,
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
