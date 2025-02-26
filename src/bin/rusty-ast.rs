use std::io;
use std::path::PathBuf;

use clap::{ArgGroup, Parser};
use rusty_ast::{JsonVisitor, TextVisitor, parse_rust_file, parse_rust_source};
use syn::visit::Visit;

/// Tool for parsing Rust code and displaying its AST
///
/// # Arguments
/// * `file`: &str - path to the rust source file
/// * `code`: &str - rust source code
/// * `format`: &str - output format (text or json)
/// * `indent`: usize - number of spaces to indent
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(group(ArgGroup::new("input").required(true).args(["file", "code"])))]
struct Cli {
    /// Path to the Rust source file to parse
    #[arg(short, long, value_name = "FILE")]
    file: Option<PathBuf>,

    /// Rust code to parse (string)
    #[arg(short, long, value_name = "CODE")]
    code: Option<String>,

    /// Output format (text or json)
    #[arg(short = 'o', long, value_enum, default_value = "text")]
    format: OutputFormat,

    /// Number of spaces to indent
    #[arg(short, long, default_value_t = 2)]
    indent: usize,
}

#[derive(clap::ValueEnum, Clone)]
enum OutputFormat {
    /// Text format (indented)
    Text,
    /// JSON format
    Json,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // parse AST from file or code string
    let ast = if let Some(file_path) = cli.file {
        parse_rust_file(file_path)?
    } else if let Some(code) = cli.code {
        parse_rust_source(&code).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
    } else {
        unreachable!("clap should require one of the arguments");
    };

    // display AST according to output format
    match cli.format {
        OutputFormat::Text => {
            println!("AST for Rust code:");
            let mut visitor = TextVisitor::new();
            visitor.visit_file(&ast);
        }
        OutputFormat::Json => {
            let mut visitor = JsonVisitor::new();
            visitor.visit_file(&ast);
            println!("{}", visitor.to_json());
        }
    }

    Ok(())
}
