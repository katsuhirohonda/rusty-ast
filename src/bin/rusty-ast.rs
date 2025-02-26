use std::io;
use std::path::{Path, PathBuf};

use clap::{ArgGroup, Parser};
use rusty_ast::{JsonVisitor, TextVisitor, parse_rust_file, parse_rust_source};
use syn::visit::Visit;
use walkdir::WalkDir;

/// Tool for parsing Rust code and displaying its AST
///
/// # Arguments
/// * `file`: &str - path to the rust source file
/// * `code`: &str - rust source code
/// * `directory`: &str - path to directory containing rust source files
/// * `format`: &str - output format (text or json)
/// * `recursive`: bool - whether to search directories recursively
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(group(ArgGroup::new("input").required(true).args(["file", "code", "directory"])))]
struct Cli {
    /// Path to the Rust source file to parse
    #[arg(short, long, value_name = "FILE")]
    file: Option<PathBuf>,

    /// Rust code to parse (string)
    #[arg(short, long, value_name = "CODE")]
    code: Option<String>,

    /// Directory containing Rust files to parse
    #[arg(short = 'd', long, value_name = "DIRECTORY")]
    directory: Option<PathBuf>,

    /// Output format (text or json)
    #[arg(short = 'o', long, value_enum, default_value = "text")]
    format: OutputFormat,

    /// Recursively process directories (only applies with --directory)
    #[arg(short = 'r', long)]
    recursive: bool,
}

#[derive(clap::ValueEnum, Clone)]
enum OutputFormat {
    /// Text format (indented)
    Text,
    /// JSON format
    Json,
}

/// Process a directory and parse all Rust files
///
/// # Arguments
/// * `directory`: &Path - path to the directory
/// * `format`: &OutputFormat - output format (text or json)
/// * `recursive`: bool - whether to search subdirectories
///
/// # Returns
/// * `io::Result<()>` - result
fn process_directory(directory: &Path, format: &OutputFormat, recursive: bool) -> io::Result<()> {
    // Counter for processed files
    let mut processed_files = 0;

    // Walk the directory
    let walker = if recursive {
        WalkDir::new(directory)
    } else {
        WalkDir::new(directory).max_depth(1)
    };

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // Only process Rust files
        if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
            println!("\n--- Processing file: {} ---", path.display());

            // Parse and analyze the file
            match parse_rust_file(path) {
                Ok(ast) => {
                    processed_files += 1;

                    // Display AST according to output format
                    match format {
                        OutputFormat::Text => {
                            println!("AST for Rust code in {}:", path.display());
                            let mut visitor = TextVisitor::new();
                            visitor.visit_file(&ast);
                        }
                        OutputFormat::Json => {
                            let mut visitor = JsonVisitor::new();
                            visitor.visit_file(&ast);
                            println!("{}", visitor.to_json());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing file {}: {}", path.display(), e);
                }
            }
        }
    }

    if processed_files == 0 {
        println!("No Rust files found in the specified directory.");
    } else {
        println!("\nProcessed {} Rust files.", processed_files);
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // If directory is specified, process it
    if let Some(directory) = cli.directory {
        process_directory(&directory, &cli.format, cli.recursive)?;
        return Ok(());
    }

    // Parse AST from file or code string (original functionality)
    let ast = if let Some(file_path) = cli.file {
        parse_rust_file(file_path)?
    } else if let Some(code) = cli.code {
        parse_rust_source(&code).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
    } else {
        unreachable!("clap should require one of the arguments");
    };

    // Display AST according to output format
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_directory_processing() {
        // Create temporary test directory
        let temp_dir = TempDir::new().unwrap();

        // Create a test Rust file
        let file_path = temp_dir.path().join("test.rs");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"fn test() { println!(\"Hello\"); }")
            .unwrap();

        // Test non-recursive directory processing
        process_directory(temp_dir.path(), &OutputFormat::Text, false).unwrap();

        // Test recursive directory processing
        let nested_dir = temp_dir.path().join("nested");
        fs::create_dir(&nested_dir).unwrap();
        let nested_file_path = nested_dir.join("nested_test.rs");
        let mut nested_file = fs::File::create(nested_file_path).unwrap();
        nested_file
            .write_all(b"fn nested_test() { return 42; }")
            .unwrap();

        process_directory(temp_dir.path(), &OutputFormat::Text, true).unwrap();
    }
}
