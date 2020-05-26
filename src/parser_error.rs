use crate::error::ReportableError;
use crate::token::{Span, Token};

/// A ReportableError originating during parsing.
#[derive(Debug)]
pub enum ParsingError {
    UnexpectedToken { expected: String, actual: Token },
    SelfInheritance { span: Span },
    UnexpectedEof { index: usize },
}

impl ReportableError for ParsingError {
    fn span(&self) -> Span {
        match self {
            ParsingError::UnexpectedToken { actual, .. } => actual.span,
            ParsingError::SelfInheritance { span, .. } => *span,
            ParsingError::UnexpectedEof { index } => Span::new(*index, index + 1),
        }
    }
    fn message(&self) -> String {
        let message = match self {
            ParsingError::UnexpectedToken {
                expected, actual, ..
            } => format!(
                "Unexpected Token. Expected {} but got {}",
                expected, actual.kind
            ),
            ParsingError::SelfInheritance { .. } => "Class cannot inherit from itself".to_string(),
            ParsingError::UnexpectedEof { .. } => "Unexpected end of file".to_string(),
        };
        format!("Parsing Error - {}", message)
    }
}
