use crate::error::ReportableError;
use crate::token::Span;

/// A ReportableError originating during compilation.
#[derive(Debug)]
pub struct CompilerError {
    pub message: String,
    pub span: Span,
}

impl ReportableError for CompilerError {
    fn span(&self) -> Span {
        self.span
    }
    fn message(&self) -> String {
        format!("Compilation Error - {}", self.message)
    }
}
