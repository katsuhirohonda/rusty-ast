use std::io;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::io::Read;

    // 標準出力の取得を補助するための構造体
    struct CaptureStdout {
        old_stdout: Option<std::io::Stdout>,
        buffer: Vec<u8>,
    }

    impl CaptureStdout {
        fn new() -> Self {
            CaptureStdout {
                old_stdout: None,
                buffer: Vec::new(),
            }
        }

        fn start(&mut self) {
            // 標準出力をリダイレクト
            // 実際のアプリケーションでは、テストではなく
            // ライブラリAPIを使用して出力を直接取得すべき
        }

        fn stop(&mut self) -> String {
            // リダイレクトを停止して出力を取得
            String::from_utf8_lossy(&self.buffer).to_string()
        }
    }

    // 基本的な関数をパースしてASTが正しく構築されるかテスト
    #[test]
    fn test_parse_function() {
        let source = r#"
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        "#;

        let file = parse_rust_source(source).unwrap();

        // ファイルにはアイテムが1つ（関数定義）含まれているはず
        assert_eq!(file.items.len(), 1);

        // アイテムが関数であることを確認
        if let syn::Item::Fn(func) = &file.items[0] {
            assert_eq!(func.sig.ident.to_string(), "add");
            assert_eq!(func.sig.inputs.len(), 2); // 2つのパラメータがあるはず

            // 戻り値の型を確認
            if let syn::ReturnType::Type(_, return_type) = &func.sig.output {
                if let syn::Type::Path(type_path) = &**return_type {
                    let path_segment = &type_path.path.segments[0];
                    assert_eq!(path_segment.ident.to_string(), "i32");
                } else {
                    panic!("Return type is not a path");
                }
            } else {
                panic!("Function has no return type");
            }

            // 関数の本体に1つのステートメントがあることを確認（a + b）
            assert_eq!(func.block.stmts.len(), 1);
        } else {
            panic!("Item is not a function");
        }
    }

    // 構造体をパースするテスト
    #[test]
    fn test_parse_struct() {
        let source = r#"
            struct Point {
                x: f64,
                y: f64,
            }
        "#;

        let file = parse_rust_source(source).unwrap();

        // ファイルにはアイテムが1つ（構造体定義）含まれているはず
        assert_eq!(file.items.len(), 1);

        // アイテムが構造体であることを確認
        if let syn::Item::Struct(struct_item) = &file.items[0] {
            assert_eq!(struct_item.ident.to_string(), "Point");

            // 構造体に2つのフィールドがあることを確認
            assert_eq!(struct_item.fields.iter().count(), 2);

            // フィールド名と型を確認
            let fields: Vec<_> = struct_item.fields.iter().collect();

            // x フィールド
            assert_eq!(fields[0].ident.as_ref().unwrap().to_string(), "x");
            if let syn::Type::Path(type_path) = &fields[0].ty {
                let path_segment = &type_path.path.segments[0];
                assert_eq!(path_segment.ident.to_string(), "f64");
            } else {
                panic!("Field x is not a path type");
            }

            // y フィールド
            assert_eq!(fields[1].ident.as_ref().unwrap().to_string(), "y");
            if let syn::Type::Path(type_path) = &fields[1].ty {
                let path_segment = &type_path.path.segments[0];
                assert_eq!(path_segment.ident.to_string(), "f64");
            } else {
                panic!("Field y is not a path type");
            }
        } else {
            panic!("Item is not a struct");
        }
    }

    // 列挙型をパースするテスト
    #[test]
    fn test_parse_enum() {
        let source = r#"
            enum Direction {
                North,
                East,
                South,
                West,
            }
        "#;

        let file = parse_rust_source(source).unwrap();

        // ファイルにはアイテムが1つ（列挙型定義）含まれているはず
        assert_eq!(file.items.len(), 1);

        // アイテムが列挙型であることを確認
        if let syn::Item::Enum(enum_item) = &file.items[0] {
            assert_eq!(enum_item.ident.to_string(), "Direction");

            // 列挙型に4つのバリアントがあることを確認
            assert_eq!(enum_item.variants.len(), 4);

            // バリアント名を確認
            let variant_names: Vec<String> = enum_item
                .variants
                .iter()
                .map(|v| v.ident.to_string())
                .collect();

            assert_eq!(variant_names, vec!["North", "East", "South", "West"]);
        } else {
            panic!("Item is not an enum");
        }
    }

    // 複雑な式をパースするテスト
    #[test]
    fn test_parse_complex_expression() {
        let source = r#"
            fn complex_expr() {
                let result = (10 + 20) * 30 / (5 - 2);
                if result > 100 {
                    println!("Large result: {}", result);
                } else {
                    println!("Small result: {}", result);
                }
            }
        "#;

        let file = parse_rust_source(source).unwrap();

        // ファイルにはアイテムが1つ（関数定義）含まれているはず
        assert_eq!(file.items.len(), 1);

        // アイテムが関数であることを確認
        if let syn::Item::Fn(func) = &file.items[0] {
            assert_eq!(func.sig.ident.to_string(), "complex_expr");

            // 関数の本体に2つのステートメントがあることを確認
            // (変数宣言とif文)
            assert_eq!(func.block.stmts.len(), 2);

            // 最初のステートメントが変数宣言であることを確認
            if let syn::Stmt::Local(local) = &func.block.stmts[0] {
                assert!(local.init.is_some());

                // 変数名がresultであることを確認
                if let syn::Pat::Ident(pat_ident) = &local.pat {
                    assert_eq!(pat_ident.ident.to_string(), "result");
                } else {
                    panic!("Variable declaration pattern is not an identifier");
                }
            } else {
                panic!("First statement is not a variable declaration");
            }

            // 2番目のステートメントがif式であることを確認
            if let syn::Stmt::Expr(expr, _) = &func.block.stmts[1] {
                if let syn::Expr::If(_) = expr {
                    // OK
                } else {
                    panic!("Second statement is not an if expression");
                }
            } else {
                panic!("Second statement is not an expression");
            }
        } else {
            panic!("Item is not a function");
        }
    }

    // 無効なコードのパースをテスト
    #[test]
    fn test_parse_invalid_code() {
        let source = r#"
            fn invalid_function( {
                let x = 10;
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_err(), "Expected parse error for invalid code");
    }

    // ASTVisitorの機能テスト
    // 注意: このテストは標準出力をキャプチャできないため、
    // 実際のプロジェクトでは機能を再設計することを推奨
    // 複数アイテム（関数、構造体など）をパースするテスト
    #[test]
    fn test_parse_multiple_items() {
        let source = r#"
            fn function1() -> i32 { 42 }
            
            struct MyStruct {
                field: i32,
            }
            
            fn function2(s: MyStruct) -> i32 {
                s.field
            }
        "#;

        let file = parse_rust_source(source).unwrap();

        // ファイルには3つのアイテムが含まれているはず
        assert_eq!(file.items.len(), 3);

        // 1番目のアイテムが関数であることを確認
        if let syn::Item::Fn(func) = &file.items[0] {
            assert_eq!(func.sig.ident.to_string(), "function1");
        } else {
            panic!("First item is not a function");
        }

        // 2番目のアイテムが構造体であることを確認
        if let syn::Item::Struct(struct_item) = &file.items[1] {
            assert_eq!(struct_item.ident.to_string(), "MyStruct");
        } else {
            panic!("Second item is not a struct");
        }

        // 3番目のアイテムが関数であることを確認
        if let syn::Item::Fn(func) = &file.items[2] {
            assert_eq!(func.sig.ident.to_string(), "function2");
        } else {
            panic!("Third item is not a function");
        }
    }
}
