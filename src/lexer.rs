#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Var,
    Val,
    Fun,
    Type,
    If,
    Else,
    True,
    False,
    Return,
    For,
    In,
    Import,
    Export,
    From,
    Native,
    Extern,
    Identifier(String),
    String(String),
    Integer(i64),
    Colon,
    ColonEqual,
    Equal,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    AndAnd,
    OrOr,
    Bang,
    Dot,
    Caret,     // ^ for pointer types and dereference
    Ampersand, // & for address-of
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    #[allow(dead_code)] // For future error reporting
    pub line: usize,
    #[allow(dead_code)] // For future error reporting
    pub column: usize,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

pub type LexerResult<T> = std::result::Result<T, String>;

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> LexerResult<Vec<Token>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();
            if self.is_at_end() {
                break;
            }

            let token = self.next_token();
            tokens.push(token);
        }

        tokens.push(Token {
            token_type: TokenType::Eof,
            line: self.line,
            column: self.column,
        });

        Ok(tokens)
    }

    fn next_token(&mut self) -> Token {
        let line = self.line;
        let column = self.column;

        let ch = self.current_char();

        let token_type = match ch {
            ':' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::ColonEqual
                } else {
                    TokenType::Colon
                }
            }
            '=' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                }
            }
            '{' => {
                self.advance();
                TokenType::LeftBrace
            }
            '}' => {
                self.advance();
                TokenType::RightBrace
            }
            '(' => {
                self.advance();
                TokenType::LeftParen
            }
            ')' => {
                self.advance();
                TokenType::RightParen
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
            ';' => {
                self.advance();
                TokenType::Semicolon
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
                // Check for comments
                if !self.is_at_end() && self.current_char() == '*' {
                    // Handle /* */ comments
                    self.advance(); // Skip the '*'
                    self.skip_block_comment();
                    // Recursively get the next token after skipping the comment
                    return self.next_token();
                } else if !self.is_at_end() && self.current_char() == '/' {
                    // Handle // line comments
                    self.skip_line_comment();
                    // Recursively get the next token after skipping the comment
                    return self.next_token();
                } else {
                    TokenType::Slash
                }
            }
            '%' => {
                self.advance();
                TokenType::Percent
            }
            '!' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::NotEqual
                } else {
                    TokenType::Bang
                }
            }
            '<' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }
            }
            '>' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            }
            '&' => {
                self.advance();
                if self.current_char() == '&' {
                    self.advance();
                    TokenType::AndAnd
                } else {
                    TokenType::Ampersand
                }
            }
            '|' => {
                self.advance();
                if self.current_char() == '|' {
                    self.advance();
                    TokenType::OrOr
                } else {
                    panic!("Unexpected character: '|'");
                }
            }
            '.' => {
                self.advance();
                TokenType::Dot
            }
            '^' => {
                self.advance();
                TokenType::Caret
            }
            '\n' => {
                self.advance();
                TokenType::Newline
            }
            '"' => self.read_string(),
            _ if ch.is_alphabetic() || ch == '_' => self.read_identifier(),
            _ if ch.is_numeric() => self.read_number(),
            _ => panic!("Unexpected character: {}", ch),
        };

        Token {
            token_type,
            line,
            column,
        }
    }

    fn read_string(&mut self) -> TokenType {
        self.advance(); // Skip opening quote
        let mut value = String::new();

        while !self.is_at_end() && self.current_char() != '"' {
            if self.current_char() == '\\' {
                self.advance();
                if !self.is_at_end() {
                    match self.current_char() {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '\\' => value.push('\\'),
                        '"' => value.push('"'),
                        _ => {
                            value.push('\\');
                            value.push(self.current_char());
                        }
                    }
                    self.advance();
                }
            } else {
                value.push(self.current_char());
                self.advance();
            }
        }

        if !self.is_at_end() {
            self.advance(); // Skip closing quote
        }

        TokenType::String(value)
    }

    fn read_identifier(&mut self) -> TokenType {
        let mut value = String::new();

        while !self.is_at_end()
            && (self.current_char().is_alphanumeric() || self.current_char() == '_')
        {
            value.push(self.current_char());
            self.advance();
        }

        match value.as_str() {
            "var" => TokenType::Var,
            "val" => TokenType::Val,
            "fun" => TokenType::Fun,
            "type" => TokenType::Type,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "return" => TokenType::Return,
            "for" => TokenType::For,
            "in" => TokenType::In,
            "import" => TokenType::Import,
            "export" => TokenType::Export,
            "from" => TokenType::From,
            "native" => TokenType::Native,
            "extern" => TokenType::Extern,
            _ => TokenType::Identifier(value),
        }
    }

    fn read_number(&mut self) -> TokenType {
        let mut value = String::new();

        while !self.is_at_end() && self.current_char().is_numeric() {
            value.push(self.current_char());
            self.advance();
        }

        TokenType::Integer(value.parse().unwrap())
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.current_char() {
                ' ' | '\r' | '\t' => self.advance(),
                _ => break,
            }
        }
    }

    fn skip_block_comment(&mut self) {
        // We've already consumed /* so we need to find the closing */
        while !self.is_at_end() {
            if self.current_char() == '*' {
                self.advance();
                if !self.is_at_end() && self.current_char() == '/' {
                    self.advance(); // Skip the closing '/'
                    return;
                }
            } else if self.current_char() == '\n' {
                self.line += 1;
                self.advance();
            } else {
                self.advance();
            }
        }

        // If we reach here, we hit EOF without finding closing */
        // This could be handled as an error in the future
    }

    fn skip_line_comment(&mut self) {
        // We've already consumed // so skip until end of line or EOF
        while !self.is_at_end() && self.current_char() != '\n' {
            self.advance();
        }
        // Don't advance past the newline - let normal processing handle it
    }

    fn current_char(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.position]
        }
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            if self.current_char() == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize_string(input: &str) -> Vec<TokenType> {
        let mut lexer = Lexer::new(input.to_string());
        lexer
            .tokenize()
            .unwrap()
            .into_iter()
            .map(|token| token.token_type)
            .collect()
    }

    #[test]
    fn test_keywords() {
        let tokens =
            tokenize_string("var val fun type if else true false return for in import export from");
        assert_eq!(
            tokens,
            vec![
                TokenType::Var,
                TokenType::Val,
                TokenType::Fun,
                TokenType::Type,
                TokenType::If,
                TokenType::Else,
                TokenType::True,
                TokenType::False,
                TokenType::Return,
                TokenType::For,
                TokenType::In,
                TokenType::Import,
                TokenType::Export,
                TokenType::From,
                TokenType::Eof
            ]
        );
    }

    #[test]
    fn test_identifiers() {
        let tokens = tokenize_string("myVar _underscore camelCase PascalCase");
        assert_eq!(tokens.len(), 5); // 4 identifiers + EOF
        assert!(matches!(tokens[0], TokenType::Identifier(_)));
        assert!(matches!(tokens[1], TokenType::Identifier(_)));
        assert!(matches!(tokens[2], TokenType::Identifier(_)));
        assert!(matches!(tokens[3], TokenType::Identifier(_)));
    }

    #[test]
    fn test_string_literals() {
        let tokens = tokenize_string(r#""hello world" "test""#);
        assert_eq!(tokens.len(), 3); // 2 strings + EOF
        if let TokenType::String(s) = &tokens[0] {
            assert_eq!(s, "hello world");
        } else {
            panic!("Expected string token");
        }
        if let TokenType::String(s) = &tokens[1] {
            assert_eq!(s, "test");
        } else {
            panic!("Expected string token");
        }
    }

    #[test]
    fn test_integer_literals() {
        let tokens = tokenize_string("42 0 999 123456789");
        assert_eq!(tokens.len(), 5); // 4 integers + EOF
        if let TokenType::Integer(n) = tokens[0] {
            assert_eq!(n, 42);
        } else {
            panic!("Expected integer token");
        }
    }

    #[test]
    fn test_operators() {
        let tokens = tokenize_string(":= : = + - * / % == != < <= > >= && || ! . ^ &");
        assert_eq!(
            tokens,
            vec![
                TokenType::ColonEqual,
                TokenType::Colon,
                TokenType::Equal,
                TokenType::Plus,
                TokenType::Minus,
                TokenType::Star,
                TokenType::Slash,
                TokenType::Percent,
                TokenType::EqualEqual,
                TokenType::NotEqual,
                TokenType::Less,
                TokenType::LessEqual,
                TokenType::Greater,
                TokenType::GreaterEqual,
                TokenType::AndAnd,
                TokenType::OrOr,
                TokenType::Bang,
                TokenType::Dot,
                TokenType::Caret,
                TokenType::Ampersand,
                TokenType::Eof
            ]
        );
    }

    #[test]
    fn test_punctuation() {
        let tokens = tokenize_string("{ } ( ) [ ] , ;");
        assert_eq!(
            tokens,
            vec![
                TokenType::LeftBrace,
                TokenType::RightBrace,
                TokenType::LeftParen,
                TokenType::RightParen,
                TokenType::LeftBracket,
                TokenType::RightBracket,
                TokenType::Comma,
                TokenType::Semicolon,
                TokenType::Eof
            ]
        );
    }

    #[test]
    fn test_comments() {
        let tokens = tokenize_string("var/*comment*/x");
        assert_eq!(
            tokens,
            vec![
                TokenType::Var,
                TokenType::Identifier("x".to_string()),
                TokenType::Eof
            ]
        );
    }

    #[test]
    fn test_whitespace_handling() {
        let tokens = tokenize_string("  var   x  \n  :=   42  ");
        assert_eq!(
            tokens,
            vec![
                TokenType::Var,
                TokenType::Identifier("x".to_string()),
                TokenType::Newline,
                TokenType::ColonEqual,
                TokenType::Integer(42),
                TokenType::Eof
            ]
        );
    }

    #[test]
    fn test_real_code_example() {
        let code = r#"
            val name := "Alice"
            var age: Integer = 30
            if (age >= 18) {
                print("Adult")
            }
        "#;
        let tokens = tokenize_string(code);

        // Should contain: val, identifier, :=, string, newline, var, identifier, :, etc.
        assert!(tokens.len() > 10);
        assert!(matches!(tokens[0], TokenType::Newline)); // Leading newline
        assert!(matches!(tokens[1], TokenType::Val));
        assert!(matches!(tokens[2], TokenType::Identifier(_)));
        assert!(matches!(tokens[3], TokenType::ColonEqual));
        assert!(matches!(tokens[4], TokenType::String(_)));
    }

    #[test]
    fn test_line_and_column_tracking() {
        let mut lexer = Lexer::new("line1\nline2\nline3".to_string());
        let tokens = lexer.tokenize().unwrap();

        // Check that tokens have proper line/column info
        assert!(tokens.iter().all(|t| t.line >= 1));
        assert!(tokens.iter().all(|t| t.column >= 1));
    }

    #[test]
    fn test_error_handling() {
        // Test that invalid characters are handled (should panic for now)
        let result = std::panic::catch_unwind(|| tokenize_string("@#$%"));
        assert!(result.is_err());
    }
}
