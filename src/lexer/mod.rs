pub mod token;

use token::{Token, TokenType};

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();
        Lexer {
            input: chars,
            position: 0,
            current_char,
            line: 1,
            column: 1,
        }
    }

    fn advance(&mut self) {
        if self.current_char == Some('\n') {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        
        self.position += 1;
        self.current_char = if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        };
    }

    fn peek(&self, offset: usize) -> Option<char> {
        let pos = self.position + offset;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        // Skip single-line comments (// ...)
        if self.current_char == Some('/') && self.peek(1) == Some('/') {
            while self.current_char.is_some() && self.current_char != Some('\n') {
                self.advance();
            }
        }
    }

    fn read_string(&mut self) -> String {
        let mut result = String::new();
        self.advance(); // Skip opening quote

        while let Some(ch) = self.current_char {
            if ch == '"' {
                self.advance(); // Skip closing quote
                break;
            } else if ch == '\\' {
                self.advance();
                match self.current_char {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('r') => result.push('\r'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    _ => result.push('\\'),
                }
                self.advance();
            } else {
                result.push(ch);
                self.advance();
            }
        }
        result
    }

    fn read_number(&mut self) -> String {
        let mut result = String::new();
        let mut has_dot = false;

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                result.push(ch);
                self.advance();
            } else if ch == '.' && !has_dot && self.peek(1).map_or(false, |c| c.is_ascii_digit()) {
                has_dot = true;
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    fn identifier_or_keyword(&mut self) -> TokenType {
        let id = self.read_identifier();
        match id.as_str() {
            "func" => TokenType::Func,
            "return" => TokenType::Return,
            "match" => TokenType::Match,
            "case" => TokenType::Case,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "null" => TokenType::Null,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "in" => TokenType::In,
            "class" => TokenType::Class,
            "extends" => TokenType::Extends,
            "new" => TokenType::New,
            _ => TokenType::Identifier(id),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        loop {
            // Skip all whitespace and comments
            loop {
                self.skip_whitespace();
                if self.current_char != Some('/') || self.peek(1) != Some('/') {
                    break;
                }
                self.skip_comment();
            }

            let token_line = self.line;
            let token_column = self.column;

            let token_type = match self.current_char {
                None => {
                    tokens.push(Token::new(TokenType::Eof, token_line, token_column));
                    break;
                }
                Some(ch) => {
                    if ch.is_alphabetic() || ch == '_' {
                        self.identifier_or_keyword()
                    } else if ch.is_ascii_digit() {
                        let num = self.read_number();
                        TokenType::Number(num)
                    } else if ch == '"' {
                        let s = self.read_string();
                        TokenType::String(s)
                    } else {
                        match ch {
                            '=' => {
                                self.advance();
                                if self.current_char == Some('=') {
                                    self.advance();
                                    TokenType::EqualEqual
                                } else if self.current_char == Some('>') {
                                    self.advance();
                                    TokenType::Arrow
                                } else {
                                    TokenType::Assign
                                }
                            }
                            '+' => {
                                self.advance();
                                TokenType::Plus
                            }
                            '-' => {
                                self.advance();
                                TokenType::Minus
                            }
                            '*' => {
                                self.advance();
                                TokenType::Star
                            }
                            '/' => {
                                self.advance();
                                TokenType::Slash
                            }
                            '!' => {
                                self.advance();
                                if self.current_char == Some('=') {
                                    self.advance();
                                    TokenType::NotEqual
                                } else {
                                    TokenType::Bang
                                }
                            }
                            '<' => {
                                self.advance();
                                if self.current_char == Some('=') {
                                    self.advance();
                                    TokenType::LessEqual
                                } else {
                                    TokenType::Less
                                }
                            }
                            '>' => {
                                self.advance();
                                if self.current_char == Some('=') {
                                    self.advance();
                                    TokenType::GreaterEqual
                                } else {
                                    TokenType::Greater
                                }
                            }
                            '&' => {
                                self.advance();
                                if self.current_char == Some('&') {
                                    self.advance();
                                    TokenType::And
                                } else {
                                    return Err(format!("Unexpected character '&' at {}:{}", token_line, token_column));
                                }
                            }
                            '|' => {
                                self.advance();
                                if self.current_char == Some('|') {
                                    self.advance();
                                    TokenType::Or
                                } else {
                                    return Err(format!("Unexpected character '|' at {}:{}", token_line, token_column));
                                }
                            }
                            '(' => {
                                self.advance();
                                TokenType::LeftParen
                            }
                            ')' => {
                                self.advance();
                                TokenType::RightParen
                            }
                            '{' => {
                                self.advance();
                                TokenType::LeftBrace
                            }
                            '}' => {
                                self.advance();
                                TokenType::RightBrace
                            }
                            '[' => {
                                self.advance();
                                TokenType::LeftBracket
                            }
                            ']' => {
                                self.advance();
                                TokenType::RightBracket
                            }
                            ',' => {
                                self.advance();
                                TokenType::Comma
                            }
                            ':' => {
                                self.advance();
                                TokenType::Colon
                            }
                            ';' => {
                                self.advance();
                                TokenType::Semicolon
                            }
                            '.' => {
                                self.advance();
                                TokenType::Dot
                            }
                            _ => {
                                return Err(format!("Unexpected character '{}' at {}:{}", ch, token_line, token_column));
                            }
                        }
                    }
                }
            };

            tokens.push(Token::new(token_type, token_line, token_column));
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut lexer = Lexer::new("= + - * /".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token_type, TokenType::Assign);
        assert_eq!(tokens[1].token_type, TokenType::Plus);
        assert_eq!(tokens[2].token_type, TokenType::Minus);
        assert_eq!(tokens[3].token_type, TokenType::Star);
        assert_eq!(tokens[4].token_type, TokenType::Slash);
    }

    #[test]
    fn test_identifier() {
        let mut lexer = Lexer::new("greeting count".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::Identifier(_)));
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("func return match case".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token_type, TokenType::Func);
        assert_eq!(tokens[1].token_type, TokenType::Return);
        assert_eq!(tokens[2].token_type, TokenType::Match);
        assert_eq!(tokens[3].token_type, TokenType::Case);
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new("\"Hello, world!\"".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::String(_)));
    }

    #[test]
    fn test_number_literal() {
        let mut lexer = Lexer::new("42 3.14".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::Number(_)));
        assert!(matches!(tokens[1].token_type, TokenType::Number(_)));
    }
}
