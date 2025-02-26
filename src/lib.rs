//! library for parsing Rust code and visualizing AST
//!
//! This crate provides tools for parsing Rust source code and displaying its abstract syntax tree (AST).

mod parser;
mod visitor;
mod json;

pub use parser::{parse_rust_file, parse_rust_source, print_ast};
pub use visitor::AstVisitor;
pub use json::{JsonVisitor, AstJson};
