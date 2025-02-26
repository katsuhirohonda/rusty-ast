# rusty-ast

A Rust Abstract Syntax Tree (AST) visualization tool. This tool parses Rust source code and displays its syntactic structure in text or JSON format.

## Features

- Generate AST from Rust source code files or strings
- Display AST in readable text format
- JSON output option
- Support for various Rust syntax elements:
  - Function definitions
  - Struct definitions
  - Enum definitions
  - Variable declarations
  - Control flow (if, while, loop)
  - Expressions (binary operations, function calls, literals, etc.)

## Installation

Install using Cargo:

```bash
cargo install rusty-ast
```

Or clone and build from this repository:

```bash
git clone https://github.com/yourusername/rusty-ast.git
cd rusty-ast
cargo build --release
```

## Usage

### Command Line Tool

Basic usage:

```bash
# Parse a Rust source file
rusty-ast -f path/to/your/file.rs

# Parse Rust code directly
rusty-ast -c "fn main() { println!(\"Hello, world!\"); }"

# Output in JSON format
rusty-ast -f path/to/your/file.rs -o json

# Change indentation size (default is 2 spaces)
rusty-ast -f path/to/your/file.rs -i 4
```

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
rusty-ast = "0.1.0"
```

Example code:

```rust
use rusty_ast::{parse_rust_source, AstVisitor};
use syn::visit::Visit;

fn main() {
    let code = r#"
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
    "#;
    
    if let Ok(ast) = parse_rust_source(code) {
        // Display AST in text format
        let mut visitor = AstVisitor::new();
        visitor.visit_file(&ast);
    }
}
```

## License

MIT

## Contributing

Contributions are welcome, including bug reports, feature requests, and pull requests.
