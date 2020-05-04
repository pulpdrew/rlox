use rlox::compiler;
use rlox::error::ErrorReporter;
use rlox::parser::Parser;
use rlox::vm::VM;
use std::io::Write;

#[derive(Debug)]
pub struct Output {
    pub contents: String,
}

impl Output {
    pub fn new() -> Self {
        Output {
            contents: String::new(),
        }
    }
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.contents.push_str(&std::str::from_utf8(buf).unwrap());

        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn run(source: String) -> (Output, Output) {
    let mut vm = VM::new();

    let mut stdout = Output::new();
    let mut stderr = Output::new();

    let mut reporter = ErrorReporter::new(source.clone(), &mut stderr);

    // Parse
    let mut parser = Parser::new(source.clone());
    let ast = match parser.parse_program() {
        Ok(ast) => ast,
        Err(errors) => {
            errors.iter().for_each(|e| reporter.report(e));
            return (stdout, stderr);
        }
    };

    // Compile
    let script = match compiler::compile(ast) {
        Ok(bin) => bin,
        Err(e) => {
            reporter.report(&e);
            return (stdout, stderr);
        }
    };

    if cfg!(feature = "disassemble") {
        script.bin.dump();
    }

    // Execute
    match vm.interpret(&script, &mut stdout) {
        Ok(_) => {}
        Err(e) => {
            reporter.report(&e);
            return (stdout, stderr);
        }
    }

    (stdout, stderr)
}
