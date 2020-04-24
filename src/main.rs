mod ast;
mod chunk;
mod compiler;
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
use vm::VM;

fn run_file(filename: &String) {
    let source = fs::read_to_string(&filename)
        .expect(format!("Failed to read source file {}", filename).as_str());

    // Parse
    let mut parser = Parser::new(source.clone());
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
    match vm.interpret(binary) {
        Ok(()) => return,
        Err(e) => eprintln!("{}", e.message),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        run_file(&args[1]);
    } else {
        eprintln!("Usage: clox [path]");
    }
}
