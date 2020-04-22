mod opcode;
mod scanner;
mod token;

use std::env;
use std::fs;
use token::Kind;

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
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        run_file(&args[1]);
    } else {
        eprintln!("Usage: clox [path]");
    }
}
