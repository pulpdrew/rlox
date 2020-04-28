use crate::token::Span;
use std::cmp;
use std::io::Write;

#[derive(Debug)]
pub struct ErrorHandler<'a, W: Write> {
    source: String,
    error_stream: &'a mut W,
}

impl<'a, W: Write> ErrorHandler<'a, W> {
    pub fn new(source: String, error_stream: &'a mut W) -> Self {
        ErrorHandler {
            source,
            error_stream,
        }
    }

    pub fn error(&self, span: &Span, message: &str) {
        eprintln!("Error: {}", message);
        self.print_underlined_source(span);
    }

    fn print_underlined_source(&self, span: &Span) {
        let mut line_start: usize = 0;
        let mut line_counter: usize = 1;
        for line in self.source.split("\n") {
            if line_start <= span.end && line_start + line.len() >= span.start {
                let underline_start = span.start - line_start;
                let underline_end = cmp::min(line.len() + 1, span.end - line_start);
                Self::print_underlined(line, line_counter, underline_start, underline_end);
            }
            line_start += line.len() + 1;
            line_counter += 1;
        }
    }

    fn print_underlined(line: &str, num: usize, start: usize, end: usize) {
        println!("{:4}: {}", num, line);

        for _ in 0..start + 6 {
            print!(" ");
        }
        for _ in start..end {
            print!("^")
        }
        println!();
    }
}
