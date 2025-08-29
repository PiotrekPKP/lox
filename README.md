# Lox Interpreter in Rust

This is a Rust implementation of the Lox programming language, inspired by the book [Crafting Interpreters](https://craftinginterpreters.com/) by Robert Nystrom.

## About

Lox is a dynamically-typed, object-oriented programming language. This project follows the structure and concepts outlined in the book, implementing both the tree-walk interpreter and the bytecode virtual machine in Rust.

## Features

- **[WIP] Tree-Walk Interpreter**: A simple and straightforward interpreter for Lox.
- **[TODO] Bytecode Virtual Machine**: A more efficient implementation of the Lox language.
- **Error Handling**: Comprehensive error reporting for syntax and runtime errors.
- **Extensible**: Easily extendable for additional features or optimizations.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) (latest stable version)

### Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/your-username/rust-lox.git
   cd rust-lox
   ```

2. Build the project:

   ```bash
   cargo build
   ```

3. Run the interpreter:
   ```bash
   cargo run
   ```

### Usage

You can run Lox scripts by passing a file to the interpreter:

```bash
cargo run path/to/script.lox
```

Alternatively, start a REPL (Read-Eval-Print Loop) by running the interpreter without arguments:

```bash
cargo run
```

## Project Structure

- `src/`: Contains the Rust source code.
- `main.lox`: The lox file used for testing the interpreter.]

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

## Acknowledgments

- [Crafting Interpreters](https://craftinginterpreters.com/)
