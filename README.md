# RLox

RLox is an interpreter for the Lox programming language. The language was designed by Robert Nystrom for his book, [Crafting Interpreters](http://craftinginterpreters.com/). After working my way through the book, I decided to implement a third version of the interpreter, written in Rust. More information about the design of RLox can be found [here](https://pulpdrew.com/rlox).

## Running the interpreter

To run the interpreter, simply clone the repository and use cargo to build and run the project.

To run a REPL:
```sh
cargo run
```

To Run a file:
```sh
cargo run [filename]
```

To run with bytecode output
```sh
cargo run --features disassemble
```

Finally, to run unit tests and end to end tests, try
```sh
cargo test
```