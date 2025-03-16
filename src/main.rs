use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Boolean(b) => write!(f, "{}", b),
            JsonValue::Number(n) => write!(f, "{}", n),
            JsonValue::String(s) => write!(f, "\"{}\"", s),
            JsonValue::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            JsonValue::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, val)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key, val)?;
                }
                write!(f, "}}")
            }
        }
    }
}

#[derive(Debug)]
pub enum JsonError {
    UnexpectedToken(String),
    UnexpectedEof,
    InvalidNumber(String),
    InvalidEscapeSequence(String),
    InvalidUnicodeSequence(String),
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JsonError::UnexpectedToken(token) => write!(f, "Unexpected token: {}", token),
            JsonError::UnexpectedEof => write!(f, "Unexpected end of file"),
            JsonError::InvalidNumber(msg) => write!(f, "Invalid number: {}", msg),
            JsonError::InvalidEscapeSequence(seq) => write!(f, "Invalid escape sequence: {}", seq),
            JsonError::InvalidUnicodeSequence(seq) => {
                write!(f, "Invalid unicode sequence: {}", seq)
            }
        }
    }
}

impl Error for JsonError {}

pub type Result<T> = std::result::Result<T, JsonError>;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]
    Colon,        // :
    Comma,        // ,
}

struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>> {
        self.skip_whitespace();

        match self.input.peek() {
            Some(&c) => {
                match c {
                    '{' => {
                        self.input.next();
                        Ok(Some(Token::LeftBrace))
                    }
                    '}' => {
                        self.input.next();
                        Ok(Some(Token::RightBrace))
                    }
                    '[' => {
                        self.input.next();
                        Ok(Some(Token::LeftBracket))
                    }
                    ']' => {
                        self.input.next();
                        Ok(Some(Token::RightBracket))
                    }
                    ':' => {
                        self.input.next();
                        Ok(Some(Token::Colon))
                    }
                    ',' => {
                        self.input.next();
                        Ok(Some(Token::Comma))
                    }
                    '"' => {
                        self.input.next(); // Skip opening quote
                        self.read_string()
                    }
                    'n' => self.read_null(),
                    't' | 'f' => self.read_boolean(),
                    '0'..='9' | '-' => self.read_number(),
                    _ => Err(JsonError::UnexpectedToken(c.to_string())),
                }
            }
            None => Ok(None),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.input.peek() {
            if c.is_whitespace() {
                self.input.next();
            } else {
                break;
            }
        }
    }

    fn read_string(&mut self) -> Result<Option<Token>> {
        let mut string = String::new();

        while let Some(c) = self.input.next() {
            match c {
                '"' => return Ok(Some(Token::String(string))),
                '\\' => {
                    match self.input.next() {
                        Some(escape_char) => {
                            match escape_char {
                                '"' => string.push('"'),
                                '\\' => string.push('\\'),
                                '/' => string.push('/'),
                                'b' => string.push('\u{0008}'),
                                'f' => string.push('\u{000C}'),
                                'n' => string.push('\n'),
                                'r' => string.push('\r'),
                                't' => string.push('\t'),
                                'u' => {
                                    // Unicode escape sequence
                                    let mut code_point = String::new();
                                    for _ in 0..4 {
                                        if let Some(hex_digit) = self.input.next() {
                                            code_point.push(hex_digit);
                                        } else {
                                            return Err(JsonError::InvalidUnicodeSequence(
                                                code_point,
                                            ));
                                        }
                                    }

                                    match u32::from_str_radix(&code_point, 16) {
                                        Ok(cp) => {
                                            if let Some(unicode_char) = std::char::from_u32(cp) {
                                                string.push(unicode_char);
                                            } else {
                                                return Err(JsonError::InvalidUnicodeSequence(
                                                    code_point,
                                                ));
                                            }
                                        }
                                        Err(_) => {
                                            return Err(JsonError::InvalidUnicodeSequence(
                                                code_point,
                                            ));
                                        }
                                    }
                                }
                                _ => {
                                    return Err(JsonError::InvalidEscapeSequence(
                                        escape_char.to_string(),
                                    ));
                                }
                            }
                        }
                        None => return Err(JsonError::UnexpectedEof),
                    }
                }
                _ => string.push(c),
            }
        }

        Err(JsonError::UnexpectedEof)
    }

    fn read_null(&mut self) -> Result<Option<Token>> {
        let expected = "null";
        self.input.next(); // Consume 'n'

        for expected_char in expected.chars().skip(1) {
            // Skip 'n' as we've already consumed it
            match self.input.next() {
                Some(c) if c == expected_char => continue,
                Some(c) => return Err(JsonError::UnexpectedToken(c.to_string())),
                None => return Err(JsonError::UnexpectedEof),
            }
        }

        Ok(Some(Token::Null))
    }

    fn read_boolean(&mut self) -> Result<Option<Token>> {
        match self.input.peek() {
            Some(&'t') => {
                let expected = "true";
                for expected_char in expected.chars() {
                    match self.input.next() {
                        Some(c) if c == expected_char => continue,
                        Some(c) => return Err(JsonError::UnexpectedToken(c.to_string())),
                        None => return Err(JsonError::UnexpectedEof),
                    }
                }

                Ok(Some(Token::Boolean(true)))
            }
            Some(&'f') => {
                let expected = "false";
                for expected_char in expected.chars() {
                    match self.input.next() {
                        Some(c) if c == expected_char => continue,
                        Some(c) => return Err(JsonError::UnexpectedToken(c.to_string())),
                        None => return Err(JsonError::UnexpectedEof),
                    }
                }

                Ok(Some(Token::Boolean(false)))
            }
            _ => Err(JsonError::UnexpectedToken(
                "Expected 'true' or 'false'".to_string(),
            )),
        }
    }

    fn read_number(&mut self) -> Result<Option<Token>> {
        let mut number_str = String::new();

        // Handle negative sign
        if let Some(&'-') = self.input.peek() {
            number_str.push(self.input.next().unwrap());
        }

        // Integer part
        self.read_digits(&mut number_str)?;

        // Fractional part
        if let Some(&'.') = self.input.peek() {
            number_str.push(self.input.next().unwrap());
            self.read_digits(&mut number_str)?;
        }

        // Exponent part
        if let Some(&c) = self.input.peek() {
            if c == 'e' || c == 'E' {
                number_str.push(self.input.next().unwrap());

                // Handle exponent sign
                if let Some(&c) = self.input.peek() {
                    if c == '+' || c == '-' {
                        number_str.push(self.input.next().unwrap());
                    }
                }

                self.read_digits(&mut number_str)?;
            }
        }

        match number_str.parse::<f64>() {
            Ok(num) => Ok(Some(Token::Number(num))),
            Err(_) => Err(JsonError::InvalidNumber(number_str)),
        }
    }

    fn read_digits(&mut self, number_str: &mut String) -> Result<()> {
        let mut has_digit = false;

        while let Some(&c) = self.input.peek() {
            if c.is_digit(10) {
                has_digit = true;
                number_str.push(self.input.next().unwrap());
            } else {
                break;
            }
        }

        if !has_digit {
            return Err(JsonError::InvalidNumber(
                "Expected at least one digit".to_string(),
            ));
        }

        Ok(())
    }
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Option<Token>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Result<Self> {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token()?;

        Ok(Parser {
            lexer,
            current_token,
        })
    }

    fn parse(&mut self) -> Result<JsonValue> {
        let value = self.parse_value()?;

        // Ensure we've consumed all tokens
        if self.current_token.is_some() {
            return Err(JsonError::UnexpectedToken(
                "Expected end of input".to_string(),
            ));
        }

        Ok(value)
    }

    fn advance_token(&mut self) -> Result<()> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }

    fn parse_value(&mut self) -> Result<JsonValue> {
        match &self.current_token {
            Some(Token::Null) => {
                self.advance_token()?;
                Ok(JsonValue::Null)
            }
            Some(Token::Boolean(b)) => {
                let value = *b;
                self.advance_token()?;
                Ok(JsonValue::Boolean(value))
            }
            Some(Token::Number(n)) => {
                let value = *n;
                self.advance_token()?;
                Ok(JsonValue::Number(value))
            }
            Some(Token::String(s)) => {
                let value = s.clone();
                self.advance_token()?;
                Ok(JsonValue::String(value))
            }
            Some(Token::LeftBrace) => self.parse_object(),
            Some(Token::LeftBracket) => self.parse_array(),
            Some(token) => Err(JsonError::UnexpectedToken(format!("{:?}", token))),
            None => Err(JsonError::UnexpectedEof),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue> {
        // Consume the opening brace
        self.advance_token()?;

        let mut object = HashMap::new();

        // Handle empty object
        if let Some(Token::RightBrace) = &self.current_token {
            self.advance_token()?;
            return Ok(JsonValue::Object(object));
        }

        loop {
            // Parse key (must be a string)
            let key = match &self.current_token {
                Some(Token::String(s)) => {
                    let key = s.clone();
                    self.advance_token()?;
                    key
                }
                Some(token) => {
                    return Err(JsonError::UnexpectedToken(format!(
                        "Expected string key, got {:?}",
                        token
                    )));
                }
                None => return Err(JsonError::UnexpectedEof),
            };

            // Parse colon
            match &self.current_token {
                Some(Token::Colon) => self.advance_token()?,
                Some(token) => {
                    return Err(JsonError::UnexpectedToken(format!(
                        "Expected ':', got {:?}",
                        token
                    )));
                }
                None => return Err(JsonError::UnexpectedEof),
            }

            // Parse value
            let value = self.parse_value()?;

            // Add key-value pair to object
            object.insert(key, value);

            // Check for comma or closing brace
            match &self.current_token {
                Some(Token::Comma) => {
                    self.advance_token()?;
                    // Handle trailing comma (not allowed in JSON)
                    if let Some(Token::RightBrace) = &self.current_token {
                        return Err(JsonError::UnexpectedToken(
                            "Trailing comma in object".to_string(),
                        ));
                    }
                }
                Some(Token::RightBrace) => {
                    self.advance_token()?;
                    break;
                }
                Some(token) => {
                    return Err(JsonError::UnexpectedToken(format!(
                        "Expected ',' or '}}', got {:?}",
                        token
                    )));
                }
                None => return Err(JsonError::UnexpectedEof),
            }
        }

        Ok(JsonValue::Object(object))
    }

    fn parse_array(&mut self) -> Result<JsonValue> {
        // Consume the opening bracket
        self.advance_token()?;

        let mut array = Vec::new();

        // Handle empty array
        if let Some(Token::RightBracket) = &self.current_token {
            self.advance_token()?;
            return Ok(JsonValue::Array(array));
        }

        loop {
            // Parse value
            let value = self.parse_value()?;

            // Add value to array
            array.push(value);

            // Check for comma or closing bracket
            match &self.current_token {
                Some(Token::Comma) => {
                    self.advance_token()?;
                    // Handle trailing comma (not allowed in JSON)
                    if let Some(Token::RightBracket) = &self.current_token {
                        return Err(JsonError::UnexpectedToken(
                            "Trailing comma in array".to_string(),
                        ));
                    }
                }
                Some(Token::RightBracket) => {
                    self.advance_token()?;
                    break;
                }
                Some(token) => {
                    return Err(JsonError::UnexpectedToken(format!(
                        "Expected ',' or ']', got {:?}",
                        token
                    )));
                }
                None => return Err(JsonError::UnexpectedEof),
            }
        }

        Ok(JsonValue::Array(array))
    }
}

fn parse_json(input: &str) -> Result<JsonValue> {
    let mut parser = Parser::new(input)?;
    parser.parse()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let json_str = if args.len() > 1 {
        // Read JSON from file
        match fs::read_to_string(&args[1]) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("File reading error: {}", e);
                return;
            }
        }
    } else {
        // Read JSON from standard input
        println!("Enter JSON (end with Ctrl+D):");
        let mut buffer = String::new();
        match io::stdin().read_to_string(&mut buffer) {
            Ok(_) => buffer,
            Err(e) => {
                eprintln!("Standard input reading error: {}", e);
                return;
            }
        }
    };

    match parse_json(&json_str) {
        Ok(json) => println!("Parsed JSON: {}", json),
        Err(e) => eprintln!("JSON parsing error: {}", e),
    }
}

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
