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
