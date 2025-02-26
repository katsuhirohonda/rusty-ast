//! library for parsing Rust code and visualizing AST
//!
//! This crate provides tools for parsing Rust source code and displaying its abstract syntax tree (AST).

mod json_visitor;
mod parser;
mod text_visitor;

pub use json_visitor::{AstJson, JsonVisitor};
pub use parser::{parse_rust_file, parse_rust_source, print_ast};
pub use text_visitor::AstVisitor;
