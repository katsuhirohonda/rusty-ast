use std::fs;
use std::io;
use std::path::PathBuf;

use clap::{ArgGroup, Parser};
use rusty_ast::{AstVisitor, parse_rust_source};

/// Rust コードを解析して AST（抽象構文木）を表示するツール
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(group(ArgGroup::new("input").required(true).args(["file", "code"])))]
struct Cli {
    /// 解析するRustファイルのパス
    #[arg(short, long, value_name = "FILE")]
    file: Option<PathBuf>,

    /// 直接解析するRustコード（文字列）
    #[arg(short, long, value_name = "CODE")]
    code: Option<String>,

    /// 出力形式（text または json）
    #[arg(short, long, value_enum, default_value = "text")]
    format: OutputFormat,

    /// インデントに使用するスペースの数
    #[arg(short, long, default_value_t = 2)]
    indent: usize,
}

#[derive(clap::ValueEnum, Clone)]
enum OutputFormat {
    /// テキスト形式（インデント付き）
    Text,
    /// JSON形式
    Json,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // ファイルまたはコード文字列からASTを解析
    let _ast = if let Some(_file_path) = cli.file {
        // parse_rust_file(file_path)?
        todo!()
    } else if let Some(code) = cli.code {
        parse_rust_source(&code).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
    } else {
        unreachable!("clap がどちらかの引数を要求するはず");
    };

    // 出力形式に応じて表示
    match cli.format {
        OutputFormat::Text => {
            println!("AST for Rust code:");
            //let mut visitor = AstVisitor::new();
            // visitor.visit_file(&ast);
            todo!()
        }
        OutputFormat::Json => {
            // 注意: 実際にはJSONシリアライズの実装が必要
            // serde_jsonなどを使用して実装する
            println!("JSON output is not implemented yet");
        }
    }

    Ok(())
}
