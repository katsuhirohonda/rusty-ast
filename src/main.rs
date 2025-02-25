use std::io;

mod parser;
mod visitor;

use parser::{parse_rust_source, print_ast};

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

    // 標準出力の取得を補助するための構造体
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
