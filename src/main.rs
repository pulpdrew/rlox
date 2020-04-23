mod chunk;
mod object;
mod opcode;
mod scanner;
mod token;
mod value;
mod vm;

#[macro_use]
extern crate num_derive;

use opcode::OpCode;
use std::env;
use std::fs;
use token::Kind;
use value::Value;
use vm::VM;

fn run_file(filename: &String) {
    let source = fs::read_to_string(&filename)
        .expect(format!("Failed to read source file {}", filename).as_str());
    let mut scanner = scanner::Scanner::new(source.clone());
    loop {
        let next = scanner.next();
        if next.kind == Kind::Eof {
            break;
        } else {
            println!("{:?}", next)
        }
    }

    let mut chunk = chunk::Chunk::new(String::from("script"));
    chunk.push_constant(Value::Number(10.0), 1);
    chunk.push_constant(Value::Number(12.0), 1);
    chunk.push_opcode(OpCode::Add, 1);
    chunk.push_opcode(OpCode::Return, 3);
    chunk.dump();

    println!("Interpreting: ");
    let mut vm = VM::new();
    match vm.interpret(chunk) {
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
