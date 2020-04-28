extern crate rlox;

use rlox::compiler::Compiler;
use rlox::error::ErrorHandler;
use rlox::parser::Parser;
use rlox::vm::VM;
use std::env;
use std::fs;
use std::io::{self, Write};

fn run(source: String, vm: &mut VM) {
    // Parse
    let handler = ErrorHandler::new(source.clone());
    let mut parser = Parser::new(source.clone(), handler);
    let ast = parser.parse_program();
    if parser.had_error {
        return;
    }

    // Compile
    let mut compiler = Compiler::new();
    let binary = compiler.compile(ast);
    if cfg!(feature = "disassemble") {
        binary.dump();
    }

    // Execute
    if cfg!(feature = "disassemble") {
        println!("Interpreting: ");
    }
    vm.interpret(binary, &ErrorHandler::new(source));
}

fn run_file(filename: &str) {
    let source = fs::read_to_string(&filename)
        .unwrap_or_else(|_| panic!("Failed to read source file {}", filename));
    let mut vm = VM::new();
    run(source, &mut vm);
}

fn repl() {
    let stdin = io::stdin();
    let mut vm = VM::new();
    loop {
        print!("> ");
        io::stdout().flush().expect("Failed to flush to output.");

        let mut source = String::new();
        loop {
            match stdin.read_line(&mut source) {
                Ok(count) => {
                    if count <= 1 {
                        run(String::from(source.trim_end()), &mut vm);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        run_file(&args[1]);
    } else if args.len() == 1 {
        repl();
    } else {
        eprintln!("Usage: clox [path]");
    }
}
