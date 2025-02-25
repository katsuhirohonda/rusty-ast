use syn::{File, visit::Visit};

use crate::visitor::AstVisitor;

// Rustのソースコード文字列からASTに変換
pub fn parse_rust_source(source: &str) -> Result<syn::File, syn::Error> {
    syn::parse_file(source)
}

// AST表示の例
pub fn print_ast(file: &File) {
    println!("AST for Rust code:");
    let mut visitor = AstVisitor::new();
    visitor.visit_file(file);
}
