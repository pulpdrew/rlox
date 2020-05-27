extern crate rlox;

use rlox::compiler;
use rlox::error::ErrorReporter;
use rlox::parser::Parser;
use rlox::vm::VM;
use std::env;
use std::fs;
use std::io::{self, Write};

fn run(source: String, vm: &mut VM) {
    let mut stderr = std::io::stderr();
    let mut reporter = ErrorReporter::new(&source, &mut stderr);

    // Parse
    let mut parser = Parser::new(&source);
    let ast = match parser.parse_program() {
        Ok(ast) => ast,
        Err(errors) => {
            errors.iter().for_each(|e| reporter.report(e));
            return;
        }
    };

    // Compile
    let script = match compiler::compile(ast) {
        Ok(bin) => bin,
        Err(e) => {
            reporter.report(&e);
            return;
        }
    };

    if cfg!(feature = "disassemble") {
        script.function.bin.dump(&mut std::io::stdout());
    }

    // Execute
    vm.reset();
    match vm.execute(&script, &mut std::io::stdout()) {
        Ok(_) => {}
        Err(e) => {
            reporter.report(&e);
        }
    }
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
            let mut buffer = String::new();
            stdin.read_line(&mut buffer).unwrap();
            if buffer.trim_end().is_empty() {
                break;
            } else {
                source.push_str(&buffer);
            }
        }

        println!("{}", source);
        run(source, &mut vm);
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
