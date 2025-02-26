use quote::ToTokens;
use serde::Serialize;
use syn::{Expr, File, Item, Lit, Pat, Stmt, visit::Visit};

/// A serializable representation of a Rust AST for JSON output
#[derive(Serialize, Debug, Default)]
pub struct AstJson {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ItemJson>,
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum ItemJson {
    Function {
        name: String,
        parameters: Vec<ParameterJson>,
        return_type: Option<String>,
        body: Vec<StmtJson>,
    },
    Struct {
        name: String,
        fields: Vec<FieldJson>,
    },
    Enum {
        name: String,
        variants: Vec<VariantJson>,
    },
    Other {
        description: String,
    },
}

#[derive(Serialize, Debug)]
pub struct ParameterJson {
    pub name: String,
    pub type_info: String,
}

#[derive(Serialize, Debug)]
pub struct FieldJson {
    pub name: Option<String>,
    pub type_info: String,
}

#[derive(Serialize, Debug)]
pub struct VariantJson {
    pub name: String,
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum StmtJson {
    VariableDeclaration {
        name: String,
        initializer: Option<Box<ExprJson>>,
    },
    Expression {
        expr: Box<ExprJson>,
    },
    Other {
        description: String,
    },
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum ExprJson {
    IntLiteral {
        value: String,
    },
    FloatLiteral {
        value: String,
    },
    StringLiteral {
        value: String,
    },
    BoolLiteral {
        value: bool,
    },
    Binary {
        operator: String,
        left: Box<ExprJson>,
        right: Box<ExprJson>,
    },
    FunctionCall {
        function: Box<ExprJson>,
        arguments: Vec<ExprJson>,
    },
    Identifier {
        name: String,
    },
    If {
        condition: Box<ExprJson>,
        then_branch: Vec<StmtJson>,
        else_branch: Option<Box<ExprJson>>,
    },
    Loop {
        body: Vec<StmtJson>,
    },
    While {
        condition: Box<ExprJson>,
        body: Vec<StmtJson>,
    },
    Return {
        value: Option<Box<ExprJson>>,
    },
    Other {
        description: String,
    },
}

/// A visitor that builds a JSON representation of a Rust AST
pub struct JsonVisitor {
    pub ast: AstJson,
}

impl JsonVisitor {
    pub fn new() -> Self {
        JsonVisitor {
            ast: AstJson::default(),
        }
    }

    pub fn to_json(&self) -> String {
        // デバッグ情報を追加
        eprintln!("AST項目数: {}", self.ast.items.len());

        if let Some(first_item) = self.ast.items.first() {
            eprintln!("最初の項目の型: {:?}", first_item);
        }

        // JSONシリアライズを試みる
        match serde_json::to_string_pretty(&self.ast) {
            Ok(json) => {
                // JSON出力の一部をデバッグとして表示
                if !json.is_empty() && json.len() > 10 {
                    let preview_len = std::cmp::min(json.len(), 100);
                    eprintln!("JSON出力プレビュー: {}...", &json[0..preview_len]);
                }
                json
            }
            Err(e) => {
                eprintln!("JSONシリアライズエラー: {}", e);
                String::from("{}")
            }
        }
    }

    // ファイル処理の専用メソッド
    pub fn process_file(&mut self, file: &File) {
        // ファイル内の各項目を処理
        for item in &file.items {
            self.process_item(item);
        }
    }

    // 項目を処理する専用メソッド
    fn process_item(&mut self, item: &Item) {
        match item {
            Item::Fn(item_fn) => {
                let mut parameters = Vec::new();
                for param in &item_fn.sig.inputs {
                    match param {
                        syn::FnArg::Typed(pat_type) => {
                            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                                parameters.push(ParameterJson {
                                    name: pat_ident.ident.to_string(),
                                    type_info: format!("{}", (*pat_type.ty).to_token_stream()),
                                });
                            }
                        }
                        syn::FnArg::Receiver(receiver) => {
                            parameters.push(ParameterJson {
                                name: "self".to_string(),
                                type_info: format!("{}", receiver.to_token_stream()),
                            });
                        }
                    }
                }

                let return_type = match &item_fn.sig.output {
                    syn::ReturnType::Default => None,
                    syn::ReturnType::Type(_, return_type) => {
                        Some(format!("{}", return_type.to_token_stream()))
                    }
                };

                let mut statements = Vec::new();
                for stmt in &item_fn.block.stmts {
                    statements.push(self.visit_stmt_json(stmt));
                }

                self.ast.items.push(ItemJson::Function {
                    name: item_fn.sig.ident.to_string(),
                    parameters,
                    return_type,
                    body: statements,
                });
            }
            Item::Struct(item_struct) => {
                let mut fields = Vec::new();
                for field in &item_struct.fields {
                    fields.push(FieldJson {
                        name: field.ident.as_ref().map(|ident| ident.to_string()),
                        type_info: format!("{}", field.ty.to_token_stream()),
                    });
                }

                self.ast.items.push(ItemJson::Struct {
                    name: item_struct.ident.to_string(),
                    fields,
                });
            }
            Item::Enum(item_enum) => {
                let mut variants = Vec::new();
                for variant in &item_enum.variants {
                    variants.push(VariantJson {
                        name: variant.ident.to_string(),
                    });
                }

                self.ast.items.push(ItemJson::Enum {
                    name: item_enum.ident.to_string(),
                    variants,
                });
            }
            _ => {
                self.ast.items.push(ItemJson::Other {
                    description: format!("{}", item.to_token_stream()),
                });
            }
        }
    }
}

impl<'ast> Visit<'ast> for JsonVisitor {
    fn visit_file(&mut self, file: &'ast File) {
        // Visit トレイトの実装を変更
        // 専用の process_file メソッドを使用してファイルを処理
        self.process_file(file);

        // 親の visit_file は呼び出さない - すでに全ての項目を処理しているため
    }
}

impl JsonVisitor {
    fn visit_stmt_json(&mut self, stmt: &Stmt) -> StmtJson {
        match stmt {
            Stmt::Local(local) => {
                let name = if let Pat::Ident(pat_ident) = &local.pat {
                    pat_ident.ident.to_string()
                } else {
                    "unknown".to_string()
                };

                let initializer = if let Some(init) = &local.init {
                    Some(Box::new(self.visit_expr_json(&init.expr)))
                } else {
                    None
                };

                StmtJson::VariableDeclaration { name, initializer }
            }
            Stmt::Expr(expr, _) => StmtJson::Expression {
                expr: Box::new(self.visit_expr_json(expr)),
            },
            _ => StmtJson::Other {
                description: format!("{}", stmt.to_token_stream()),
            },
        }
    }

    fn visit_expr_json(&mut self, expr: &Expr) -> ExprJson {
        match expr {
            Expr::Lit(expr_lit) => match &expr_lit.lit {
                Lit::Int(lit_int) => ExprJson::IntLiteral {
                    value: lit_int.base10_digits().to_string(),
                },
                Lit::Float(lit_float) => ExprJson::FloatLiteral {
                    value: lit_float.base10_digits().to_string(),
                },
                Lit::Str(lit_str) => ExprJson::StringLiteral {
                    value: lit_str.value(),
                },
                Lit::Bool(lit_bool) => ExprJson::BoolLiteral {
                    value: lit_bool.value,
                },
                _ => ExprJson::Other {
                    description: format!("{}", expr_lit.to_token_stream()),
                },
            },
            Expr::Binary(expr_bin) => {
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

                ExprJson::Binary {
                    operator: op.to_string(),
                    left: Box::new(self.visit_expr_json(&expr_bin.left)),
                    right: Box::new(self.visit_expr_json(&expr_bin.right)),
                }
            }
            Expr::Call(expr_call) => ExprJson::FunctionCall {
                function: Box::new(self.visit_expr_json(&expr_call.func)),
                arguments: expr_call
                    .args
                    .iter()
                    .map(|arg| self.visit_expr_json(arg))
                    .collect(),
            },
            Expr::Path(expr_path) => ExprJson::Identifier {
                name: format!("{}", expr_path.to_token_stream()),
            },
            Expr::If(expr_if) => {
                let mut then_stmts = Vec::new();
                for stmt in &expr_if.then_branch.stmts {
                    then_stmts.push(self.visit_stmt_json(stmt));
                }

                let else_branch = if let Some((_, else_expr)) = &expr_if.else_branch {
                    Some(Box::new(self.visit_expr_json(else_expr)))
                } else {
                    None
                };

                ExprJson::If {
                    condition: Box::new(self.visit_expr_json(&expr_if.cond)),
                    then_branch: then_stmts,
                    else_branch,
                }
            }
            Expr::Loop(expr_loop) => {
                let mut stmts = Vec::new();
                for stmt in &expr_loop.body.stmts {
                    stmts.push(self.visit_stmt_json(stmt));
                }

                ExprJson::Loop { body: stmts }
            }
            Expr::While(expr_while) => {
                let mut stmts = Vec::new();
                for stmt in &expr_while.body.stmts {
                    stmts.push(self.visit_stmt_json(stmt));
                }

                ExprJson::While {
                    condition: Box::new(self.visit_expr_json(&expr_while.cond)),
                    body: stmts,
                }
            }
            Expr::Return(expr_return) => ExprJson::Return {
                value: expr_return
                    .expr
                    .as_ref()
                    .map(|e| Box::new(self.visit_expr_json(e))),
            },
            _ => ExprJson::Other {
                description: format!("{}", expr.to_token_stream()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_rust_source;

    #[test]
    fn test_json_serialization_function() {
        let source = r#"
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        "#;

        let file = parse_rust_source(source).unwrap();
        let mut visitor = JsonVisitor::new();
        visitor.process_file(&file);

        let json = visitor.to_json();
        eprintln!("関数テスト - JSON出力: {}", json);

        // JSONの構造を見ると、nameはitemsの中の要素に含まれている
        assert!(json.contains("\"items\":["));
        assert!(json.contains("\"type\":\"Function\""));
        assert!(json.contains("\"name\":\"add\""));
        assert!(json.contains("\"parameters\":["));
        assert!(json.contains("\"return_type\":\"i32\""));
        assert!(json.contains("\"operator\":\"+\""));
    }

    #[test]
    fn test_json_serialization_struct() {
        let source = r#"
            struct Point {
                x: f64,
                y: f64,
            }
        "#;

        let file = parse_rust_source(source).unwrap();
        let mut visitor = JsonVisitor::new();
        visitor.process_file(&file);

        let json = visitor.to_json();
        eprintln!("構造体テスト - JSON出力: {}", json);

        // JSONの構造を見ると、nameはitemsの中の要素に含まれている
        assert!(json.contains("\"items\":["));
        assert!(json.contains("\"type\":\"Struct\""));
        assert!(json.contains("\"name\":\"Point\""));
        assert!(json.contains("\"fields\":["));
        assert!(json.contains("\"name\":\"x\""));
        assert!(json.contains("\"type_info\":\"f64\""));
    }

    #[test]
    fn test_json_serialization_enum() {
        let source = r#"
            enum Direction {
                North,
                East,
                South,
                West,
            }
        "#;

        let file = parse_rust_source(source).unwrap();
        let mut visitor = JsonVisitor::new();
        visitor.process_file(&file);

        let json = visitor.to_json();
        eprintln!("列挙型テスト - JSON出力: {}", json);

        // JSONの構造を見ると、nameはitemsの中の要素に含まれている
        assert!(json.contains("\"items\":["));
        assert!(json.contains("\"type\":\"Enum\""));
        assert!(json.contains("\"name\":\"Direction\""));
        assert!(json.contains("\"variants\":["));
        assert!(json.contains("\"name\":\"North\""));
        assert!(json.contains("\"name\":\"East\""));
        assert!(json.contains("\"name\":\"South\""));
        assert!(json.contains("\"name\":\"West\""));
    }

    #[test]
    fn test_json_serialization_complex() {
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
        let mut visitor = JsonVisitor::new();
        visitor.process_file(&file);

        let json = visitor.to_json();
        eprintln!("複雑な式テスト - JSON出力: {}", json);

        // JSONの構造を見ると、nameはitemsの中の要素に含まれている
        assert!(json.contains("\"items\":["));
        assert!(json.contains("\"type\":\"Function\""));
        assert!(json.contains("\"name\":\"complex_expr\""));
        assert!(json.contains("\"type\":\"VariableDeclaration\""));
        assert!(json.contains("\"name\":\"result\""));
        assert!(json.contains("\"type\":\"If\""));
        assert!(json.contains("\"operator\":\">\""));
    }

    // 追加のデバッグテスト
    #[test]
    fn test_debug_json_output() {
        // 簡単なソースコード
        let source = r#"
            fn test_func() {
                println!("Hello");
            }
        "#;

        // パースと訪問処理
        let file = parse_rust_source(source).unwrap();
        let mut visitor = JsonVisitor::new();
        visitor.process_file(&file);

        // AST構造を詳細に出力
        eprintln!("AST構造: {:#?}", visitor.ast);

        // JSON化
        let json = visitor.to_json();
        eprintln!("JSON出力: {}", json);

        // 検証 - 項目が空でないこと
        assert!(!visitor.ast.items.is_empty(), "AST項目が空です！");
        // 正しい検証 - 構造に合わせて items 配列内の要素をチェック
        assert!(json.contains("\"items\":["));
        assert!(json.contains("\"type\":\"Function\""));
        assert!(json.contains("\"name\":\"test_func\""));
    }

    // 基本的なシリアライズのテスト
    #[test]
    fn test_basic_serialization() {
        // 手動でAstJson構造を作成
        let mut ast = AstJson::default();

        // 関数アイテムを追加
        ast.items.push(ItemJson::Function {
            name: "manual_func".to_string(),
            parameters: vec![],
            return_type: Some("i32".to_string()),
            body: vec![],
        });

        // JSONシリアライズ
        let json = serde_json::to_string_pretty(&ast).unwrap();
        eprintln!("手動作成したJSONシリアライズ: {}", json);

        // 正しい検証 - 構造に合わせて items 配列内の要素をチェック
        assert!(json.contains("\"items\":["));
        assert!(json.contains("\"type\":\"Function\""));
        assert!(json.contains("\"name\":\"manual_func\""));
    }
}
