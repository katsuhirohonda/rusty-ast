use quote::ToTokens;
use serde::Serialize;
use syn::{Expr, File, Item, Lit, Pat, Stmt, visit::Visit};

/// A serializable representation of a Rust AST for JSON output
///
/// # Fields
/// * `items`: Vec<ItemJson> - the items in the AST
#[derive(Serialize, Debug, Default)]
pub struct AstJson {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ItemJson>,
}

/// # Fields
/// * `name`: String - the name of the item
/// * `parameters`: Vec<ParameterJson> - the parameters of the item
/// * `return_type`: Option<String> - the return type of the item
/// * `body`: Vec<StmtJson> - the body of the item
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

/// # Fields
/// * `name`: String - the name of the parameter
/// * `type_info`: String - the type of the parameter
#[derive(Serialize, Debug)]
pub struct ParameterJson {
    pub name: String,
    pub type_info: String,
}

/// # Fields
/// * `name`: Option<String> - the name of the field
/// * `type_info`: String - the type of the field
#[derive(Serialize, Debug)]
pub struct FieldJson {
    pub name: Option<String>,
    pub type_info: String,
}

/// # Fields
/// * `name`: String - the name of the variant
#[derive(Serialize, Debug)]
pub struct VariantJson {
    pub name: String,
}

/// # Fields
/// * `name`: String - the name of the statement
/// * `initializer`: Option<Box<ExprJson>> - the initializer of the statement
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

/// # Fields
/// * `value`: String - the value of the literal
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
///
/// # Fields
/// * `ast`: AstJson - the AST to be converted to JSON
pub struct JsonVisitor {
    pub ast: AstJson,
}

/// # Methods
/// * `new()`: creates a new JsonVisitor
/// * `to_json()`: converts the AST to a JSON string
/// * `process_file()`: processes a file and adds its items to the AST
/// * `process_item()`: processes an item and adds it to the AST
impl Default for JsonVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonVisitor {
    /// new
    ///
    /// # Arguments
    /// * `()`
    ///
    /// # Returns
    /// * `JsonVisitor` - a new JsonVisitor
    pub fn new() -> Self {
        JsonVisitor {
            ast: AstJson::default(),
        }
    }

    /// to_json
    ///
    /// # Arguments
    /// * `self`: &Self - the JsonVisitor
    ///
    /// # Returns
    /// * `String` - the JSON string
    pub fn to_json(&self) -> String {
        match serde_json::to_string_pretty(&self.ast) {
            Ok(json) => json,
            Err(_) => String::from("{}"),
        }
    }

    /// process_file
    ///
    /// # Arguments
    /// * `self`: &mut Self - the JsonVisitor
    /// * `file`: &File - the file to process
    ///
    /// # Returns
    /// * `()`
    pub fn process_file(&mut self, file: &File) {
        for item in &file.items {
            self.process_item(item);
        }
    }

    /// process_item
    ///
    /// # Arguments
    /// * `self`: &mut Self - the JsonVisitor
    /// * `item`: &Item - the item to process
    ///
    /// # Returns
    /// * `()`
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

/// # Implementations
/// * `Visit<'ast>` - the Visit trait
impl<'ast> Visit<'ast> for JsonVisitor {
    /// visit_file
    ///
    /// # Arguments
    /// * `self`: &mut Self - the JsonVisitor
    /// * `file`: &File - the file to process
    ///
    /// # Returns
    /// * `()`
    fn visit_file(&mut self, file: &'ast File) {
        self.process_file(file);
    }
}

/// # Implementations
/// * `Visit<'ast>` - the Visit trait
impl JsonVisitor {
    /// visit_stmt_json
    ///
    /// # Arguments
    /// * `self`: &mut Self - the JsonVisitor
    /// * `stmt`: &Stmt - the statement to process
    ///
    /// # Returns
    fn visit_stmt_json(&mut self, stmt: &Stmt) -> StmtJson {
        match stmt {
            Stmt::Local(local) => {
                let name = if let Pat::Ident(pat_ident) = &local.pat {
                    pat_ident.ident.to_string()
                } else {
                    "unknown".to_string()
                };

                let initializer = local.init.as_ref().map(|init| Box::new(self.visit_expr_json(&init.expr)));

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

    /// visit_expr_json
    ///
    /// # Arguments
    /// * `self`: &mut Self - the JsonVisitor
    /// * `expr`: &Expr - the expression to process
    ///
    /// # Returns
    /// * `ExprJson` - the JSON representation of the expression
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
    use serde_json::Value;

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

        // JSONをパースして構造を確認
        let parsed: Value = serde_json::from_str(&json).expect("JSONのパースに失敗");

        // 構造を確認 - itemsは配列
        assert!(parsed["items"].is_array());

        // 最初の要素を取得
        let first_item = &parsed["items"][0];

        // 型と名前を確認
        assert_eq!(first_item["type"], "Function");
        assert_eq!(first_item["name"], "add");

        // パラメータを確認
        assert!(first_item["parameters"].is_array());
        assert_eq!(first_item["parameters"][0]["name"], "a");
        assert_eq!(first_item["parameters"][1]["name"], "b");

        // 戻り値を確認
        assert_eq!(first_item["return_type"], "i32");
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

        // JSONをパースして構造を確認
        let parsed: Value = serde_json::from_str(&json).expect("JSONのパースに失敗");

        // 構造を確認
        assert!(parsed["items"].is_array());

        // 最初の要素を取得
        let first_item = &parsed["items"][0];

        // 型と名前を確認
        assert_eq!(first_item["type"], "Struct");
        assert_eq!(first_item["name"], "Point");

        // フィールドを確認
        assert!(first_item["fields"].is_array());
        assert_eq!(first_item["fields"][0]["name"], "x");
        assert_eq!(first_item["fields"][1]["name"], "y");
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

        // JSONをパースして構造を確認
        let parsed: Value = serde_json::from_str(&json).expect("JSONのパースに失敗");

        // 構造を確認
        assert!(parsed["items"].is_array());

        // 最初の要素を取得
        let first_item = &parsed["items"][0];

        // 型と名前を確認
        assert_eq!(first_item["type"], "Enum");
        assert_eq!(first_item["name"], "Direction");

        // バリアントを確認
        assert!(first_item["variants"].is_array());
        assert_eq!(first_item["variants"][0]["name"], "North");
        assert_eq!(first_item["variants"][1]["name"], "East");
        assert_eq!(first_item["variants"][2]["name"], "South");
        assert_eq!(first_item["variants"][3]["name"], "West");
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

        // JSONをパースして構造を確認
        let parsed: Value = serde_json::from_str(&json).expect("JSONのパースに失敗");

        // 構造を確認
        assert!(parsed["items"].is_array());

        // 最初の要素を取得
        let first_item = &parsed["items"][0];

        // 型と名前を確認
        assert_eq!(first_item["type"], "Function");
        assert_eq!(first_item["name"], "complex_expr");

        // 関数本体の文を確認
        assert!(first_item["body"].is_array());
        assert_eq!(first_item["body"][0]["type"], "VariableDeclaration");
        assert_eq!(first_item["body"][0]["name"], "result");

        // if文を確認
        assert_eq!(first_item["body"][1]["type"], "Expression");
        assert_eq!(first_item["body"][1]["expr"]["type"], "If");
    }

    // 追加のデバッグテスト
    #[test]
    fn test_debug_json_output() {
        let source = r#"
            fn test_func() {
                println!("Hello");
            }
        "#;

        let file = parse_rust_source(source).unwrap();
        let mut visitor = JsonVisitor::new();
        visitor.process_file(&file);

        // JSON化
        let json = visitor.to_json();

        // 検証 - 項目が空でないこと
        assert!(!visitor.ast.items.is_empty(), "AST項目が空です！");

        // JSONをパースして構造を確認
        let parsed: Value = serde_json::from_str(&json).expect("JSONのパースに失敗");

        // 構造を確認
        assert!(parsed["items"].is_array());

        // 最初の要素を取得
        let first_item = &parsed["items"][0];

        // 型と名前を確認
        assert_eq!(first_item["type"], "Function");
        assert_eq!(first_item["name"], "test_func");
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

        // JSONをパースして構造を確認
        let parsed: Value = serde_json::from_str(&json).expect("JSONのパースに失敗");

        // 構造を確認
        assert!(parsed["items"].is_array());

        // 最初の要素を取得
        let first_item = &parsed["items"][0];

        // 型と名前を確認
        assert_eq!(first_item["type"], "Function");
        assert_eq!(first_item["name"], "manual_func");
    }
}
