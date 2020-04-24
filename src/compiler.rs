use crate::ast::{Expression, Statement};
use crate::chunk::Chunk;
use crate::token::Kind;
use crate::vm::OpCode;

#[derive(Debug)]
pub struct Compiler {}

impl Compiler {
    pub fn new() -> Self {
        Compiler {}
    }
    pub fn compile(&mut self, program: Statement) -> Chunk {
        let mut chunk = Chunk::new(String::from("script"));
        self.compile_statement(&mut chunk, program);
        chunk.push_opcode(OpCode::Return, *chunk.lines.last().unwrap_or(&0));
        chunk
    }

    fn compile_statement(&mut self, chunk: &mut Chunk, statement: Statement) {
        match statement {
            Statement::Expression { expression, semi } => {
                self.compile_expression(chunk, *expression);
                chunk.push_opcode(OpCode::Pop, semi.line);
            }
            Statement::None => panic!("Cannot compile invalid ast."),
        }
    }

    fn compile_expression(&mut self, chunk: &mut Chunk, expression: Expression) {
        match expression {
            Expression::Constant { value, literal } => {
                chunk.push_constant(value, literal.line);
            }
            Expression::Unary {
                operator,
                expression,
            } => {
                self.compile_expression(chunk, *expression);
                chunk.push_opcode(OpCode::Negate, operator.line);
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
                    _ => panic!("Invalid binary operator {:?}", operator),
                }
            }
            Expression::None => panic!("Cannot compile invalid ast."),
        }
    }
}
