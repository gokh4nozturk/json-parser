pub mod error;
pub mod json;
pub mod lexer;
pub mod parser;

// Re-export main types for easier access
pub use error::{JsonError, Result};
pub use json::JsonValue;
pub use parser::parse_json;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let input = "null";
        let result = parse_json(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), JsonValue::Null);
    }

    #[test]
    fn test_parse_boolean() {
        let input = "true";
        let result = parse_json(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), JsonValue::Boolean(true));

        let input = "false";
        let result = parse_json(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), JsonValue::Boolean(false));
    }

    #[test]
    fn test_parse_number() {
        let input = "123";
        let result = parse_json(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), JsonValue::Number(123.0));

        let input = "-123.456";
        let result = parse_json(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), JsonValue::Number(-123.456));

        let input = "1.23e4";
        let result = parse_json(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), JsonValue::Number(12300.0));
    }

    #[test]
    fn test_parse_string() {
        let input = r#""hello""#;
        let result = parse_json(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), JsonValue::String("hello".to_string()));

        let input = r#""hello\nworld""#;
        let result = parse_json(input);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            JsonValue::String("hello\nworld".to_string())
        );
    }

    #[test]
    fn test_parse_array() {
        let input = "[1, 2, 3]";
        let result = parse_json(input);
        assert!(result.is_ok());

        if let JsonValue::Array(arr) = result.unwrap() {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], JsonValue::Number(1.0));
            assert_eq!(arr[1], JsonValue::Number(2.0));
            assert_eq!(arr[2], JsonValue::Number(3.0));
        } else {
            panic!("Expected array");
        }

        let input = r#"[1, "hello", true, null]"#;
        let result = parse_json(input);
        assert!(result.is_ok());

        if let JsonValue::Array(arr) = result.unwrap() {
            assert_eq!(arr.len(), 4);
            assert_eq!(arr[0], JsonValue::Number(1.0));
            assert_eq!(arr[1], JsonValue::String("hello".to_string()));
            assert_eq!(arr[2], JsonValue::Boolean(true));
            assert_eq!(arr[3], JsonValue::Null);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_parse_object() {
        let input = r#"{"name": "John", "age": 30}"#;
        let result = parse_json(input);
        assert!(result.is_ok());

        if let JsonValue::Object(obj) = result.unwrap() {
            assert_eq!(obj.len(), 2);
            assert_eq!(
                obj.get("name"),
                Some(&JsonValue::String("John".to_string()))
            );
            assert_eq!(obj.get("age"), Some(&JsonValue::Number(30.0)));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_nested() {
        let input = r#"
        {
            "name": "John",
            "age": 30,
            "is_student": false,
            "hobbies": ["coding", "reading", "gaming"],
            "address": {
                "city": "Istanbul",
                "country": "Turkey"
            }
        }
        "#;

        let result = parse_json(input);
        assert!(result.is_ok());

        if let JsonValue::Object(obj) = result.unwrap() {
            assert_eq!(obj.len(), 5);

            // Check hobbies array
            if let Some(JsonValue::Array(hobbies)) = obj.get("hobbies") {
                assert_eq!(hobbies.len(), 3);
                assert_eq!(hobbies[0], JsonValue::String("coding".to_string()));
                assert_eq!(hobbies[1], JsonValue::String("reading".to_string()));
                assert_eq!(hobbies[2], JsonValue::String("gaming".to_string()));
            } else {
                panic!("Expected hobbies array");
            }

            // Check address object
            if let Some(JsonValue::Object(address)) = obj.get("address") {
                assert_eq!(address.len(), 2);
                assert_eq!(
                    address.get("city"),
                    Some(&JsonValue::String("Istanbul".to_string()))
                );
                assert_eq!(
                    address.get("country"),
                    Some(&JsonValue::String("Turkey".to_string()))
                );
            } else {
                panic!("Expected address object");
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_error_cases() {
        // Invalid JSON
        let input = "{";
        let result = parse_json(input);
        assert!(result.is_err());

        // Trailing comma
        let input = "[1, 2, 3,]";
        let result = parse_json(input);
        assert!(result.is_err());

        // Missing colon in object
        let input = r#"{"name" "John"}"#;
        let result = parse_json(input);
        assert!(result.is_err());
    }
}
