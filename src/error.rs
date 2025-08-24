use std::fmt;

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

// Parser Errors
#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: String,
        location: Option<SourceLocation>,
    },
    UnexpectedEof {
        expected: String,
        location: Option<SourceLocation>,
    },
    InvalidSyntax {
        message: String,
        location: Option<SourceLocation>,
    },
    NotImplemented {
        feature: String,
        location: Option<SourceLocation>,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { expected, found, location } => {
                if let Some(loc) = location {
                    write!(f, "{}: Expected {}, found {}", loc, expected, found)
                } else {
                    write!(f, "Expected {}, found {}", expected, found)
                }
            }
            ParseError::UnexpectedEof { expected, location } => {
                if let Some(loc) = location {
                    write!(f, "{}: Unexpected end of file, expected {}", loc, expected)
                } else {
                    write!(f, "Unexpected end of file, expected {}", expected)
                }
            }
            ParseError::InvalidSyntax { message, location } => {
                if let Some(loc) = location {
                    write!(f, "{}: {}", loc, message)
                } else {
                    write!(f, "{}", message)
                }
            }
            ParseError::NotImplemented { feature, location } => {
                if let Some(loc) = location {
                    write!(f, "{}: {} not yet implemented", loc, feature)
                } else {
                    write!(f, "{} not yet implemented", feature)
                }
            }
        }
    }
}

impl std::error::Error for ParseError {}

// Lexer Errors
#[derive(Debug)]
pub enum LexError {
    UnexpectedCharacter {
        character: char,
        location: SourceLocation,
    },
    UnterminatedString {
        location: SourceLocation,
    },
    UnterminatedComment {
        location: SourceLocation,
    },
    InvalidNumber {
        value: String,
        location: SourceLocation,
    },
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LexError::UnexpectedCharacter { character, location } => {
                write!(f, "{}: Unexpected character '{}'", location, character)
            }
            LexError::UnterminatedString { location } => {
                write!(f, "{}: Unterminated string literal", location)
            }
            LexError::UnterminatedComment { location } => {
                write!(f, "{}: Unterminated comment", location)
            }
            LexError::InvalidNumber { value, location } => {
                write!(f, "{}: Invalid number '{}'", location, value)
            }
        }
    }
}

impl std::error::Error for LexError {}

// Compilation Errors
#[derive(Debug)]
pub enum CompileError {
    ParseError(ParseError),
    LexError(LexError),
    CodegenError(String),
    IoError(std::io::Error),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompileError::ParseError(e) => write!(f, "Parse error: {}", e),
            CompileError::LexError(e) => write!(f, "Lexical error: {}", e),
            CompileError::CodegenError(msg) => write!(f, "Code generation error: {}", msg),
            CompileError::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for CompileError {}

impl From<ParseError> for CompileError {
    fn from(err: ParseError) -> Self {
        CompileError::ParseError(err)
    }
}

impl From<LexError> for CompileError {
    fn from(err: LexError) -> Self {
        CompileError::LexError(err)
    }
}

impl From<std::io::Error> for CompileError {
    fn from(err: std::io::Error) -> Self {
        CompileError::IoError(err)
    }
}

// Result types
pub type ParseResult<T> = Result<T, ParseError>;
pub type LexResult<T> = Result<T, LexError>;
pub type CompileResult<T> = Result<T, CompileError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_source_location_display() {
        let loc = SourceLocation { line: 10, column: 5 };
        assert_eq!(format!("{}", loc), "10:5");
    }

    #[test]
    fn test_parse_error_display() {
        let error = ParseError::UnexpectedToken {
            expected: "identifier".to_string(),
            found: "number".to_string(),
            location: Some(SourceLocation { line: 5, column: 10 }),
        };
        let display = format!("{}", error);
        assert!(display.contains("5:10"));
        assert!(display.contains("identifier"));
        assert!(display.contains("number"));
    }

    #[test]
    fn test_parse_error_without_location() {
        let error = ParseError::UnexpectedEof {
            expected: "closing brace".to_string(),
            location: None,
        };
        let display = format!("{}", error);
        assert!(display.contains("Unexpected end of file"));
        assert!(display.contains("closing brace"));
    }

    #[test]
    fn test_lex_error_display() {
        let error = LexError::UnexpectedCharacter {
            character: '@',
            location: SourceLocation { line: 2, column: 3 },
        };
        let display = format!("{}", error);
        assert!(display.contains("2:3"));
        assert!(display.contains("@"));
        assert!(display.contains("Unexpected character"));
    }

    #[test]
    fn test_compile_error_from_parse_error() {
        let parse_error = ParseError::InvalidSyntax {
            message: "Invalid syntax".to_string(),
            location: None,
        };
        let compile_error = CompileError::from(parse_error);
        match compile_error {
            CompileError::ParseError(_) => {},
            _ => panic!("Expected ParseError variant"),
        }
    }

    #[test]
    fn test_compile_error_from_io_error() {
        let io_error = IoError::new(ErrorKind::NotFound, "File not found");
        let compile_error = CompileError::from(io_error);
        match compile_error {
            CompileError::IoError(_) => {},
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_compile_error_display() {
        let compile_error = CompileError::CodegenError("Failed to generate code".to_string());
        let display = format!("{}", compile_error);
        assert!(display.contains("Code generation error"));
        assert!(display.contains("Failed to generate code"));
    }

    #[test]
    fn test_error_trait_implementation() {
        let parse_error = ParseError::NotImplemented {
            feature: "async functions".to_string(),
            location: None,
        };
        
        // Test that it implements std::error::Error
        let _: &dyn std::error::Error = &parse_error;
    }
}