use crate::ast::{AstNode, Expression, Statement};
use crate::executable::Executable;
use crate::object::Obj;
use crate::token::{Kind, Span};
use crate::value::Value;
use crate::vm::OpCode;

#[derive(Debug)]
pub struct Compiler {}

impl Compiler {
    pub fn new() -> Self {
        Compiler {}
    }
    pub fn compile(&mut self, program: Vec<AstNode>) -> Executable {
        let mut chunk = Executable::new(String::from("script"));
        for node in program {
            self.compile_statement(&mut chunk, node.statement());
        }

        chunk.push_opcode(
            OpCode::Return,
            *chunk.spans.last().unwrap_or(&Span::new(0, 0)),
        );
        chunk
    }

    fn compile_statement(&mut self, chunk: &mut Executable, statement: &Statement) {
        match statement {
            Statement::Expression { expression } => {
                self.compile_expression(chunk, expression.expression());
                chunk.push_opcode(OpCode::Pop, expression.span);
            }
            Statement::Print {
                keyword,
                expression,
                ..
            } => {
                self.compile_expression(chunk, expression.expression());
                chunk.push_opcode(OpCode::Print, keyword.span);
            }
            Statement::Declaration {
                name,
                operator,
                initializer,
            } => {
                let name_value = Value::Obj(Obj::from(name.string.clone()));
                chunk.push_constant_inst(OpCode::DeclareGlobal, name_value.clone(), name.span);
                if let Some(init_expression) = initializer {
                    self.compile_expression(chunk, init_expression.expression());
                    chunk.push_constant_inst(
                        OpCode::SetGlobal,
                        name_value,
                        operator.as_ref().unwrap().span,
                    );
                    chunk.push_opcode(OpCode::Pop, operator.as_ref().unwrap().span)
                }
            }
        }
    }

    fn compile_expression(&mut self, chunk: &mut Executable, expression: &Expression) {
        match expression {
            Expression::Constant { value, literal } => {
                chunk.push_constant_inst(OpCode::Constant, value.clone(), literal.span);
            }
            Expression::Unary {
                operator,
                expression,
            } => {
                self.compile_expression(chunk, expression.expression());
                match operator.kind {
                    Kind::Minus => chunk.push_opcode(OpCode::Negate, operator.span),
                    Kind::Bang => chunk.push_opcode(OpCode::Not, operator.span),
                    _ => panic!("Invalid unary operator {:?}", operator),
                }
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                self.compile_expression(chunk, left.expression());
                self.compile_expression(chunk, right.expression());
                match operator.kind {
                    Kind::Plus => chunk.push_opcode(OpCode::Add, operator.span),
                    Kind::Minus => chunk.push_opcode(OpCode::Subtract, operator.span),
                    Kind::Star => chunk.push_opcode(OpCode::Multiply, operator.span),
                    Kind::Slash => chunk.push_opcode(OpCode::Divide, operator.span),
                    Kind::Less => chunk.push_opcode(OpCode::Less, operator.span),
                    Kind::LessEqual => chunk.push_opcode(OpCode::LessEqual, operator.span),
                    Kind::Greater => chunk.push_opcode(OpCode::Greater, operator.span),
                    Kind::GreaterEqual => chunk.push_opcode(OpCode::GreaterEqual, operator.span),
                    Kind::EqualEqual => chunk.push_opcode(OpCode::Equal, operator.span),
                    Kind::BangEqual => {
                        chunk.push_opcode(OpCode::Equal, operator.span);
                        chunk.push_opcode(OpCode::Not, operator.span);
                    }
                    _ => panic!("Invalid binary operator {:?}", operator),
                }
            }
            Expression::Assignment {
                lvalue,
                operator,
                rvalue,
            } => {
                if let Expression::Variable { name } = lvalue.expression() {
                    self.compile_expression(chunk, rvalue.expression());
                    let name_value = Value::Obj(Obj::from(name.string.clone()));
                    chunk.push_constant_inst(OpCode::SetGlobal, name_value, operator.span);
                } else {
                    panic!("Assignment to non-lvalue {:?}", lvalue);
                }
            }
            Expression::Variable { name } => {
                let name_value = Value::Obj(Obj::from(name.string.clone()));
                chunk.push_constant_inst(OpCode::GetGlobal, name_value, name.span);
            }
        }
    }
}
