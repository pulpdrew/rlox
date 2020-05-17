use crate::error::ReportableError;
use crate::token::Span;

/// A ReportableError originating at runtime.
#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub span: Span,
}

impl ReportableError for RuntimeError {
    fn span(&self) -> Span {
        self.span
    }
    fn message(&self) -> String {
        format!("Runtime Error - {}", self.message)
    }
}
