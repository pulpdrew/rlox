use crate::ast::{Expression, Statement};
use crate::executable::Executable;
use crate::token::Kind;
use crate::vm::OpCode;

#[derive(Debug)]
pub struct Compiler {}

impl Compiler {
    pub fn new() -> Self {
        Compiler {}
    }
    pub fn compile(&mut self, program: Vec<Statement>) -> Executable {
        let mut chunk = Executable::new(String::from("script"));
        for statement in program {
            self.compile_statement(&mut chunk, statement);
        }

        chunk.push_opcode(OpCode::Return, *chunk.lines.last().unwrap_or(&0));
        chunk
    }

    fn compile_statement(&mut self, chunk: &mut Executable, statement: Statement) {
        match statement {
            Statement::Expression { expression, semi } => {
                self.compile_expression(chunk, *expression);
                chunk.push_opcode(OpCode::Pop, semi.line);
            }
            Statement::Print {
                keyword,
                expression,
                ..
            } => {
                self.compile_expression(chunk, *expression);
                chunk.push_opcode(OpCode::Print, keyword.line);
            }
            Statement::None => panic!("Cannot compile invalid ast."),
        }
    }

    fn compile_expression(&mut self, chunk: &mut Executable, expression: Expression) {
        match expression {
            Expression::Constant { value, literal } => {
                chunk.push_constant(value, literal.line);
            }
            Expression::Unary {
                operator,
                expression,
            } => {
                self.compile_expression(chunk, *expression);
                match operator.kind {
                    Kind::Minus => chunk.push_opcode(OpCode::Negate, operator.line),
                    Kind::Bang => chunk.push_opcode(OpCode::Not, operator.line),
                    _ => panic!("Invalid unary operator {:?}", operator),
                }
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                self.compile_expression(chunk, *left);
                self.compile_expression(chunk, *right);
                match operator.kind {
                    Kind::Plus => chunk.push_opcode(OpCode::Add, operator.line),
                    Kind::Minus => chunk.push_opcode(OpCode::Subtract, operator.line),
                    Kind::Star => chunk.push_opcode(OpCode::Multiply, operator.line),
                    Kind::Slash => chunk.push_opcode(OpCode::Divide, operator.line),
                    Kind::Less => chunk.push_opcode(OpCode::Less, operator.line),
                    Kind::LessEqual => chunk.push_opcode(OpCode::LessEqual, operator.line),
                    Kind::Greater => chunk.push_opcode(OpCode::Greater, operator.line),
                    Kind::GreaterEqual => chunk.push_opcode(OpCode::GreaterEqual, operator.line),
                    Kind::EqualEqual => chunk.push_opcode(OpCode::Equal, operator.line),
                    Kind::BangEqual => {
                        chunk.push_opcode(OpCode::Equal, operator.line);
                        chunk.push_opcode(OpCode::Not, operator.line);
                    }
                    _ => panic!("Invalid binary operator {:?}", operator),
                }
            }
            Expression::True { literal } => chunk.push_opcode(OpCode::True, literal.line),
            Expression::False { literal } => chunk.push_opcode(OpCode::False, literal.line),
            Expression::None => panic!("Cannot compile invalid ast."),
        }
    }
}
