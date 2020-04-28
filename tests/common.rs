use rlox::compiler::Compiler;
use rlox::error::ErrorHandler;
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

    // Parse
    let handler = ErrorHandler::new(source.clone(), &mut stderr);
    let mut parser = Parser::new(source.clone(), &handler);
    let ast = parser.parse_program();
    if parser.had_error {
        return (stdout, stderr);
    }

    // Compile
    let mut compiler = Compiler::new();
    let binary = compiler.compile(ast);

    // Execute
    let handler = ErrorHandler::new(source, &mut stderr);
    match vm.interpret(binary, &mut stdout) {
        Ok(_) => {}
        Err(e) => {
            handler.error(&e.span, &e.message);
        }
    }

    (stdout, stderr)
}
