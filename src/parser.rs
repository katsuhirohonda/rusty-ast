use std::fs;
use std::io;
use std::path::Path;

use syn::{File, visit::Visit};

use crate::visitor::AstVisitor;

/// parse rust source code to ast
///
/// # Arguments
/// * `source`: &str - rust source code
///
/// # Returns
/// * `Result<syn::File, syn::Error>` - ast
///
/// # Errors
/// * `syn::Error` - parse error
pub fn parse_rust_source(source: &str) -> Result<syn::File, syn::Error> {
    syn::parse_file(source)
}

/// Parse Rust source code from a file into an AST
///
/// # Arguments
/// * `path`: impl AsRef<Path> - path to the rust source file
///
/// # Returns
/// * `io::Result<syn::File>` - ast
///
/// # Errors
/// * `io::Error` - file read error
/// * `syn::Error` - parse error (wrapped in io::Error)
pub fn parse_rust_file<P: AsRef<Path>>(path: P) -> io::Result<syn::File> {
    // ファイルからコードを読み込む
    let source = fs::read_to_string(path)?;

    // ソースコードをパースしてASTを生成
    let syntax =
        syn::parse_file(&source).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

    Ok(syntax)
}

/// print ast
///
/// # Arguments
/// * `file`: &File - ast
///
/// # Returns
/// * `()`
pub fn print_ast(file: &File) {
    println!("AST for Rust code:");
    let mut visitor = AstVisitor::new();
    visitor.visit_file(file);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_rust_file() {
        // 一時ファイルを作成
        let mut file = NamedTempFile::new().unwrap();
        let test_code = r#"
            fn test_function() {
                println!("Hello, world!");
            }
        "#;

        // ファイルにコードを書き込む
        file.write_all(test_code.as_bytes()).unwrap();
        file.flush().unwrap();

        // ファイルからパース
        let ast = parse_rust_file(file.path()).unwrap();

        // 基本的な検証
        assert_eq!(ast.items.len(), 1);
        if let syn::Item::Fn(func) = &ast.items[0] {
            assert_eq!(func.sig.ident.to_string(), "test_function");
        } else {
            panic!("Parsed item is not a function");
        }
    }

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
