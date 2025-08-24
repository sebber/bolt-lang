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