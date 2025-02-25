# rusty-ast

A tool for parsing Rust code and visualizing its Abstract Syntax Tree (AST).

[![Crates.io](https://img.shields.io/crates/v/rusty-ast.svg)](https://crates.io/crates/rusty-ast)
[![Documentation](https://docs.rs/rusty-ast/badge.svg)](https://docs.rs/rusty-ast)
[![License](https://img.shields.io/crates/l/rusty-ast.svg)](LICENSE)

## Features

- Parse Rust source files or code strings
- Visualize the AST structure in text format
- Display details of functions, structs, enums, and expressions
- Command-line tool with flexible options

## Installation

```sh
cargo install rusty-ast
```

## Usage

### CLI Tool

The `rusty-ast` command line tool can be used to parse and display the AST of Rust code.

```sh
# Parse a Rust file
rusty-ast --file src/main.rs

# Parse Rust code from a string
rusty-ast --code "fn main() { println!(\"Hello, world!\"); }"

# Control indentation
rusty-ast --file src/main.rs --indent 4

# Output in JSON format (coming soon)
rusty-ast --file src/main.rs --format json
```

### As a Library

```rust
use rusty_ast::{parse_rust_file, parse_rust_source, AstVisitor};
use syn::visit::Visit;

fn main() -> std::io::Result<()> {
    // Parse from a file
    let ast = parse_rust_file("src/main.rs")?;
    
    // Or parse from a string
    let code = r#"
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
    "#;
    let ast = parse_rust_source(code).unwrap();
    
    // Visualize the AST
    let mut visitor = AstVisitor::new();
    visitor.visit_file(&ast);
    
    Ok(())
}
```

## AST Structure

The tool displays Rust code structure with detailed information:

- Functions with parameters and return types
- Structs and their fields
- Enums and their variants
- Expressions (binary, function calls, conditionals, etc.)
- Literals (integers, floats, strings, booleans)
- Statements (variable declarations, expressions)

## Requirements

- Rust 1.56 or higher

## Dependencies

- syn: For parsing Rust code
- quote: For converting to token streams
- clap: For command-line argument parsing

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Here are some ways you can contribute:

1. Implement JSON output format
2. Add more detailed AST visualization
3. Add support for more Rust language features
4. Improve documentation and examples

Please feel free to submit issues and pull requests.

## Acknowledgments

- Built on top of the excellent [syn](https://crates.io/crates/syn) crate
- Inspired by tools like AST Explorer