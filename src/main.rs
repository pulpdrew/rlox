mod ast;
mod compiler;
mod error;
mod executable;
mod object;
mod parser;
mod scanner;
mod token;
mod value;
mod vm;

#[macro_use]
extern crate num_derive;

use compiler::Compiler;
use parser::Parser;
use std::env;
use std::fs;
use std::io::{self, Write};
use vm::VM;

fn run(source: String) {
    // Parse
    let handler = error::ErrorHandler::new(source.clone());
    let mut parser = Parser::new(source.clone(), handler);
    let ast = parser.parse_program();
    if parser.had_error {
        return;
    }

    // Compile
    let mut compiler = Compiler::new();
    let binary = compiler.compile(ast);
    binary.dump();

    // Execute
    println!("Interpreting: ");
    let mut vm = VM::new();
    vm.interpret(binary, &error::ErrorHandler::new(source));
}

fn run_file(filename: &str) {
    let source = fs::read_to_string(&filename)
        .unwrap_or_else(|_| panic!("Failed to read source file {}", filename));
    run(source);
}

fn repl() {
    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().expect("Failed to flush to output.");

        let mut source = String::new();
        loop {
            match stdin.read_line(&mut source) {
                Ok(count) => {
                    if count <= 1 {
                        run(String::from(source.trim_end()));
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
