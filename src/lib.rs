//! library for parsing Rust code and visualizing AST
//!
//! This crate provides tools for parsing Rust source code and displaying its abstract syntax tree (AST).

mod json_visitor;
mod text_visitor;

pub use json_visitor::{AstJson, JsonVisitor};
pub use text_visitor::{TextVisitor, parse_rust_file, parse_rust_source, print_ast};
