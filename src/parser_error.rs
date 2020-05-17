use crate::error::ReportableError;
use crate::token::Span;

/// A ReportableError originating during parsing.
#[derive(Debug)]
pub struct ParsingError {
    pub message: String,
    pub span: Span,
}

impl ReportableError for ParsingError {
    fn span(&self) -> Span {
        self.span
    }
    fn message(&self) -> String {
        format!("Parsing Error - {}", self.message)
    }
}
