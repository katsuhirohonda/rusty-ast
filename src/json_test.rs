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
        visitor.visit_file(&file);

        let json = visitor.to_json();
        
        // Basic validation - should contain function name
        assert!(json.contains("\"name\":\"add\""));
        // Should contain parameter information
        assert!(json.contains("\"parameters\":["));
        // Should contain return type
        assert!(json.contains("\"return_type\":\"i32\""));
        // Should have binary expression in body
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
        visitor.visit_file(&file);

        let json = visitor.to_json();
        
        // Basic validation
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
        visitor.visit_file(&file);

        let json = visitor.to_json();
        
        // Basic validation
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
        visitor.visit_file(&file);

        let json = visitor.to_json();
        
        // Basic validation for complex expressions
        assert!(json.contains("\"name\":\"complex_expr\""));
        assert!(json.contains("\"type\":\"VariableDeclaration\""));
        assert!(json.contains("\"name\":\"result\""));
        assert!(json.contains("\"type\":\"If\""));
        assert!(json.contains("\"operator\":\">\""));
    }
}
