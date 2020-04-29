use crate::token::Span;
use std::cmp;
use std::io::Write;

pub trait RLoxError {
    fn span(&self) -> Span;
    fn message(&self) -> String;
}

#[derive(Debug)]
pub struct ErrorReporter<'a, W: Write> {
    source: String,
    error_stream: &'a mut W,
}

impl<'a, W: 'a + Write> ErrorReporter<'a, W> {
    pub fn new(source: String, error_stream: &'a mut W) -> Self {
        ErrorReporter {
            source,
            error_stream,
        }
    }

    pub fn report<E: RLoxError>(&mut self, error: &E) {
        writeln!(self.error_stream, "{}", error.message()).unwrap();
        Self::print_underlined_source(&self.source, self.error_stream, &error.span());
    }

    fn print_underlined_source<T: Write>(source: &str, error_stream: &mut T, span: &Span) {
        let mut line_start: usize = 0;
        let mut line_num: usize = 1;
        for line in source.split('\n') {
            if line_start <= span.end && line_start + line.len() >= span.start {
                let underline_start = span.start - line_start;
                let underline_end = cmp::min(line.len() + 1, span.end - line_start);
                Self::print_underlined(
                    error_stream,
                    line,
                    line_num,
                    underline_start,
                    underline_end,
                );
            }
            line_start += line.len() + 1;
            line_num += 1;
        }
    }

    fn print_underlined<T: Write>(
        error_stream: &mut T,
        line: &str,
        line_num: usize,
        start: usize,
        end: usize,
    ) {
        writeln!(error_stream, "{:4}: {}", line_num, line).unwrap();

        for _ in 0..start + 6 {
            write!(error_stream, " ").unwrap();
        }
        for _ in start..end {
            write!(error_stream, "^").unwrap();
        }
        writeln!(error_stream).unwrap();
    }
}
