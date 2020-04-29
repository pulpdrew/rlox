use crate::ast::{AstNode, Expression, Statement};
use crate::executable::Executable;
use crate::token::{Kind, Span};
use crate::value::Value;
use crate::vm::OpCode;

#[derive(Debug, Default)]
pub struct Compiler {}

impl Compiler {
    pub fn new() -> Self {
        Compiler {}
    }
    pub fn compile(&mut self, program: Vec<AstNode>) -> Executable {
        let mut bin = Executable::new(String::from("script"));
        for node in program {
            self.compile_statement(&mut bin, &node);
        }

        bin.push_opcode(
            OpCode::Return,
            *bin.spans.last().unwrap_or(&Span::new(0, 0)),
        );
        bin
    }

    fn compile_statement(&mut self, bin: &mut Executable, statement_node: &AstNode) {
        match statement_node.statement() {
            Statement::Expression { expression } => {
                self.compile_expression(bin, expression);
                bin.push_opcode(OpCode::Pop, expression.span);
            }
            Statement::Print { expression, .. } => {
                self.compile_expression(bin, expression);
                bin.push_opcode(OpCode::Print, statement_node.span);
            }
            Statement::Declaration {
                name, initializer, ..
            } => {
                let name_value = Value::from(name.string.clone());
                bin.push_constant_inst(
                    OpCode::DeclareGlobal,
                    name_value.clone(),
                    statement_node.span,
                );
                if let Some(init_expression) = initializer {
                    self.compile_expression(bin, init_expression);
                    bin.push_constant_inst(OpCode::SetGlobal, name_value, statement_node.span);
                    bin.push_opcode(OpCode::Pop, statement_node.span)
                }
            }
        }
    }

    fn compile_expression(&mut self, bin: &mut Executable, expression_node: &AstNode) {
        match expression_node.expression() {
            Expression::Constant { value, literal } => {
                bin.push_constant_inst(OpCode::Constant, value.clone(), literal.span);
            }
            Expression::Unary {
                operator,
                expression,
            } => {
                self.compile_expression(bin, expression);
                match operator.kind {
                    Kind::Minus => bin.push_opcode(OpCode::Negate, expression_node.span),
                    Kind::Bang => bin.push_opcode(OpCode::Not, expression_node.span),
                    _ => panic!("Invalid unary operator {:?}", operator),
                }
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                self.compile_expression(bin, left);
                self.compile_expression(bin, right);
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
                    _ => panic!("Invalid binary operator {:?}", operator),
                }
            }
            Expression::Assignment { lvalue, rvalue, .. } => {
                if let Expression::Variable { name } = lvalue.expression() {
                    self.compile_expression(bin, rvalue);
                    let name_value = Value::from(name.string.clone());
                    bin.push_constant_inst(OpCode::SetGlobal, name_value, expression_node.span);
                } else {
                    panic!("Assignment to non-lvalue {:?}", lvalue);
                }
            }
            Expression::Variable { name } => {
                let name_value = Value::from(name.string.clone());
                bin.push_constant_inst(OpCode::GetGlobal, name_value, name.span);
            }
        }
    }
}
