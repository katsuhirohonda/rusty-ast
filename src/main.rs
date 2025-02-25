use std::fs;
use std::io;
use std::path::Path;

use quote::ToTokens;
use syn::{File, visit::Visit};

// ASTノードを訪問するためのビジターパターン実装
struct AstVisitor {
    indent: usize,
}

impl AstVisitor {
    fn new() -> Self {
        AstVisitor { indent: 0 }
    }

    fn print_indent(&self) -> String {
        " ".repeat(self.indent)
    }

    // visit_with_name関数を修正 - T::visit を直接呼ぶことはできない
    fn visit_with_name<'ast, T>(&mut self, node: &T, name: &str)
    where
        Self: Visit<'ast>,
    {
        println!("{}{}:", self.print_indent(), name);
        self.indent += 2;
        // 実際の処理は具体的な型に対して個別に実装する必要がある
        // ここではジェネリックな呼び出しをサポートしない
        self.indent -= 2;
    }
}

impl<'ast> syn::visit::Visit<'ast> for AstVisitor {
    // Rustの関数定義を訪問
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        println!("{}Function: {}", self.print_indent(), node.sig.ident);
        self.indent += 2;

        // 関数の引数を表示
        if !node.sig.inputs.is_empty() {
            println!("{}Parameters:", self.print_indent());
            self.indent += 2;
            for param in &node.sig.inputs {
                match param {
                    syn::FnArg::Typed(pat_type) => {
                        if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                            println!(
                                "{}Parameter: {} - Type: {}",
                                self.print_indent(),
                                pat_ident.ident,
                                pat_type.ty.to_token_stream()
                            );
                        }
                    }
                    syn::FnArg::Receiver(receiver) => {
                        println!(
                            "{}Self receiver: {}",
                            self.print_indent(),
                            receiver.to_token_stream()
                        );
                    }
                }
            }
            self.indent -= 2;
        }

        // 関数の戻り値型を表示
        if let syn::ReturnType::Type(_, return_type) = &node.sig.output {
            println!(
                "{}Return type: {}",
                self.print_indent(),
                return_type.to_token_stream()
            );
        }

        // 関数本体を訪問
        println!("{}Body:", self.print_indent());
        self.indent += 2;
        for stmt in &node.block.stmts {
            self.visit_stmt(stmt);
        }
        self.indent -= 4;
    }

    // 式を訪問
    fn visit_expr(&mut self, node: &'ast syn::Expr) {
        match node {
            syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
                syn::Lit::Int(lit_int) => {
                    println!(
                        "{}Integer literal: {}",
                        self.print_indent(),
                        lit_int.base10_digits()
                    );
                }
                syn::Lit::Float(lit_float) => {
                    println!(
                        "{}Float literal: {}",
                        self.print_indent(),
                        lit_float.base10_digits()
                    );
                }
                syn::Lit::Str(lit_str) => {
                    println!(
                        "{}String literal: \"{}\"",
                        self.print_indent(),
                        lit_str.value()
                    );
                }
                syn::Lit::Bool(lit_bool) => {
                    println!("{}Boolean literal: {}", self.print_indent(), lit_bool.value);
                }
                _ => {
                    println!(
                        "{}Other literal: {}",
                        self.print_indent(),
                        expr_lit.to_token_stream()
                    );
                }
            },
            syn::Expr::Binary(expr_bin) => {
                let op = match expr_bin.op {
                    syn::BinOp::Add(_) => "+",
                    syn::BinOp::Sub(_) => "-",
                    syn::BinOp::Mul(_) => "*",
                    syn::BinOp::Div(_) => "/",
                    syn::BinOp::Eq(_) => "==",
                    syn::BinOp::Lt(_) => "<",
                    syn::BinOp::Le(_) => "<=",
                    syn::BinOp::Ne(_) => "!=",
                    syn::BinOp::Ge(_) => ">=",
                    syn::BinOp::Gt(_) => ">",
                    _ => "other_operator",
                };
                println!("{}Binary expression: {}", self.print_indent(), op);

                println!("{}Left:", self.print_indent());
                self.indent += 2;
                self.visit_expr(&expr_bin.left);
                self.indent -= 2;

                println!("{}Right:", self.print_indent());
                self.indent += 2;
                self.visit_expr(&expr_bin.right);
                self.indent -= 2;
            }
            syn::Expr::Call(expr_call) => {
                println!("{}Function call:", self.print_indent());

                println!("{}Function:", self.print_indent());
                self.indent += 2;
                self.visit_expr(&expr_call.func);
                self.indent -= 2;

                if !expr_call.args.is_empty() {
                    println!("{}Arguments:", self.print_indent());
                    self.indent += 2;
                    for arg in &expr_call.args {
                        self.visit_expr(arg);
                    }
                    self.indent -= 2;
                }
            }
            syn::Expr::Path(expr_path) => {
                println!(
                    "{}Identifier: {}",
                    self.print_indent(),
                    expr_path.to_token_stream()
                );
            }
            syn::Expr::If(expr_if) => {
                println!("{}If statement:", self.print_indent());

                println!("{}Condition:", self.print_indent());
                self.indent += 2;
                self.visit_expr(&expr_if.cond);
                self.indent -= 2;

                println!("{}Then branch:", self.print_indent());
                self.indent += 2;
                for stmt in &expr_if.then_branch.stmts {
                    self.visit_stmt(stmt);
                }
                self.indent -= 2;

                if let Some((_, else_branch)) = &expr_if.else_branch {
                    println!("{}Else branch:", self.print_indent());
                    self.indent += 2;
                    self.visit_expr(&else_branch);
                    self.indent -= 2;
                }
            }
            syn::Expr::Loop(expr_loop) => {
                println!("{}Loop:", self.print_indent());
                self.indent += 2;
                for stmt in &expr_loop.body.stmts {
                    self.visit_stmt(stmt);
                }
                self.indent -= 2;
            }
            syn::Expr::While(expr_while) => {
                println!("{}While loop:", self.print_indent());

                println!("{}Condition:", self.print_indent());
                self.indent += 2;
                self.visit_expr(&expr_while.cond);
                self.indent -= 2;

                println!("{}Body:", self.print_indent());
                self.indent += 2;
                for stmt in &expr_while.body.stmts {
                    self.visit_stmt(stmt);
                }
                self.indent -= 2;
            }
            syn::Expr::Return(expr_return) => {
                println!("{}Return statement:", self.print_indent());
                if let Some(expr) = &expr_return.expr {
                    self.indent += 2;
                    self.visit_expr(expr);
                    self.indent -= 2;
                }
            }
            _ => {
                println!(
                    "{}Other expression: {}",
                    self.print_indent(),
                    node.to_token_stream()
                );
            }
        }
    }

    // 文を訪問 - syn::Stmtの定義が変更されたため、パターンマッチングを修正
    fn visit_stmt(&mut self, node: &'ast syn::Stmt) {
        match node {
            syn::Stmt::Local(local) => {
                println!("{}Variable declaration:", self.print_indent());
                if let syn::Pat::Ident(pat_ident) = &local.pat {
                    println!("{}Name: {}", self.print_indent(), pat_ident.ident);
                }

                // LocalInit構造体の変更に対応
                if let Some(init) = &local.init {
                    println!("{}Initializer:", self.print_indent());
                    self.indent += 2;
                    self.visit_expr(&init.expr);
                    self.indent -= 2;
                }
            }
            // Exprバリアントが変更されている - 2つのフィールドを持つようになった
            syn::Stmt::Expr(expr, _) => {
                println!("{}Expression statement:", self.print_indent());
                self.indent += 2;
                self.visit_expr(expr);
                self.indent -= 2;
            }
            // Semiバリアントが削除されたため、この分岐は不要
            syn::Stmt::Item(item) => match item {
                syn::Item::Fn(item_fn) => {
                    self.visit_item_fn(item_fn);
                }
                syn::Item::Struct(item_struct) => {
                    println!("{}Struct: {}", self.print_indent(), item_struct.ident);
                    if !item_struct.fields.is_empty() {
                        println!("{}Fields:", self.print_indent());
                        self.indent += 2;
                        for field in &item_struct.fields {
                            if let Some(ident) = &field.ident {
                                println!(
                                    "{}Field: {} - Type: {}",
                                    self.print_indent(),
                                    ident,
                                    field.ty.to_token_stream()
                                );
                            } else {
                                println!(
                                    "{}Tuple field: {}",
                                    self.print_indent(),
                                    field.ty.to_token_stream()
                                );
                            }
                        }
                        self.indent -= 2;
                    }
                }
                syn::Item::Enum(item_enum) => {
                    println!("{}Enum: {}", self.print_indent(), item_enum.ident);
                    if !item_enum.variants.is_empty() {
                        println!("{}Variants:", self.print_indent());
                        self.indent += 2;
                        for variant in &item_enum.variants {
                            println!("{}Variant: {}", self.print_indent(), variant.ident);
                        }
                        self.indent -= 2;
                    }
                }
                _ => {
                    println!(
                        "{}Other item: {}",
                        self.print_indent(),
                        item.to_token_stream()
                    );
                }
            },
            // 新しい分岐を追加（必要に応じて）
            _ => {
                println!(
                    "{}Other statement: {}",
                    self.print_indent(),
                    node.to_token_stream()
                );
            }
        }
    }
}

// Rustコードをファイルから読み込んでASTに変換
fn parse_rust_file<P: AsRef<Path>>(path: P) -> io::Result<syn::File> {
    let source = fs::read_to_string(path)?;
    let syntax =
        syn::parse_file(&source).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

    Ok(syntax)
}

// Rustのソースコード文字列からASTに変換
fn parse_rust_source(source: &str) -> Result<syn::File, syn::Error> {
    syn::parse_file(source)
}

// AST表示の例
fn print_ast(file: &File) {
    println!("AST for Rust code:");
    let mut visitor = AstVisitor::new();
    visitor.visit_file(file);
}

fn main() -> io::Result<()> {
    // 例1: ファイルからRustコードを読み込んでASTを表示
    // let path = "path/to/your/rust/file.rs";
    // let file = parse_rust_file(path)?;
    // print_ast(&file);

    // 例2: 文字列からRustコードをASTに変換して表示
    let source_code = r#"
        fn fibonacci(n: u32) -> u32 {
            if n <= 1 {
                return n;
            }
            
            let mut a = 0;
            let mut b = 1;
            
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }
            
            b
        }

        struct Point {
            x: f64,
            y: f64,
        }

        impl Point {
            fn new(x: f64, y: f64) -> Self {
                Point { x, y }
            }

            fn distance(&self, other: &Point) -> f64 {
                let dx = self.x - other.x;
                let dy = self.y - other.y;
                (dx * dx + dy * dy).sqrt()
            }
        }
    "#;

    match parse_rust_source(source_code) {
        Ok(file) => {
            print_ast(&file);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error parsing Rust code: {}", e);
            Err(io::Error::new(io::ErrorKind::InvalidData, e))
        }
    }
}
