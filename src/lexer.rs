use crate::error::{JsonError, Result};
use std::iter::Peekable;
use std::str::Chars;

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

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
        }
    }

    pub fn next_token(&mut self) -> Result<Option<Token>> {
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
