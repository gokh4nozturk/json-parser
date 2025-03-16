use crate::error::{JsonError, Result};
use crate::json::JsonValue;
use crate::lexer::{Lexer, Token};
use std::collections::HashMap;

pub struct Parser<'a> {
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
                    )))
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
                    )))
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
                    )))
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
                    )))
                }
                None => return Err(JsonError::UnexpectedEof),
            }
        }

        Ok(JsonValue::Array(array))
    }
}

/// Parse a JSON string into a JsonValue
pub fn parse_json(input: &str) -> Result<JsonValue> {
    let mut parser = Parser::new(input)?;
    parser.parse()
}
