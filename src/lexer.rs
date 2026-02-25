//! Lexer for the Martial DSL
//!
//! Tokenizes `.martial` files into a stream of tokens.

use std::fmt;

/// A token in the Martial DSL
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Roles,
    State,
    Sequence,
    Group,
    
    // Identifiers
    Identifier(String),
    
    // Symbols
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    Colon,          // :
    Arrow,          // ->
    Comma,          // ,
    
    // End of file
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Roles => write!(f, "roles"),
            Token::State => write!(f, "state"),
            Token::Sequence => write!(f, "sequence"),
            Token::Group => write!(f, "group"),
            Token::Identifier(s) => write!(f, "{}", s),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::Colon => write!(f, ":"),
            Token::Arrow => write!(f, "->"),
            Token::Comma => write!(f, ","),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

/// Position in source code for error reporting
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// A token with its position in the source
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedToken {
    pub token: Token,
    pub position: Position,
}

/// Lexer error
#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub message: String,
    pub position: Position,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lexer error at {}: {}", self.position, self.message)
    }
}

/// Lexer for the Martial DSL
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    /// Create a new lexer from input string
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    /// Get current position
    fn current_position(&self) -> Position {
        Position {
            line: self.line,
            column: self.column,
        }
    }
    
    /// Peek at current character without consuming
    fn peek(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }
    
    /// Peek at next character without consuming
    fn peek_next(&self) -> Option<char> {
        if self.position + 1 < self.input.len() {
            Some(self.input[self.position + 1])
        } else {
            None
        }
    }
    
    /// Consume and return current character
    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.peek() {
            self.position += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }
    
    /// Skip whitespace
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    /// Skip single-line comment
    fn skip_comment(&mut self) {
        // Skip the //
        self.advance();
        self.advance();
        
        // Skip until end of line
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }
    
    /// Lex an identifier or keyword
    fn lex_identifier(&mut self) -> Result<Token, LexError> {
        let mut result = String::new();
        
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        // Check if it's a keyword
        let token = match result.as_str() {
            "roles" => Token::Roles,
            "state" => Token::State,
            "sequence" => Token::Sequence,
            "group" => Token::Group,
            _ => Token::Identifier(result),
        };
        
        Ok(token)
    }
    
    /// Get the next token
    pub fn next_token(&mut self) -> Result<PositionedToken, LexError> {
        // Skip whitespace and comments
        loop {
            self.skip_whitespace();
            
            // Check for comment
            if self.peek() == Some('/') && self.peek_next() == Some('/') {
                self.skip_comment();
            } else {
                break;
            }
        }
        
        let position = self.current_position();
        
        // Check for EOF
        let ch = match self.peek() {
            Some(c) => c,
            None => return Ok(PositionedToken {
                token: Token::Eof,
                position,
            }),
        };
        
        // Single character tokens
        let token = match ch {
            '{' => {
                self.advance();
                Token::LeftBrace
            }
            '}' => {
                self.advance();
                Token::RightBrace
            }
            '[' => {
                self.advance();
                Token::LeftBracket
            }
            ']' => {
                self.advance();
                Token::RightBracket
            }
            ':' => {
                self.advance();
                Token::Colon
            }
            ',' => {
                self.advance();
                Token::Comma
            }
            '-' => {
                self.advance();
                if self.peek() == Some('>') {
                    self.advance();
                    Token::Arrow
                } else {
                    return Err(LexError {
                        message: format!("Expected '>' after '-', got {:?}", self.peek()),
                        position,
                    });
                }
            }
            _ if ch.is_alphabetic() || ch == '_' => {
                self.lex_identifier()?
            }
            _ => {
                return Err(LexError {
                    message: format!("Unexpected character: '{}'", ch),
                    position,
                });
            }
        };
        
        Ok(PositionedToken { token, position })
    }
    
    /// Tokenize the entire input
    pub fn tokenize(&mut self) -> Result<Vec<PositionedToken>, LexError> {
        let mut tokens = Vec::new();
        
        loop {
            let positioned_token = self.next_token()?;
            let is_eof = positioned_token.token == Token::Eof;
            tokens.push(positioned_token);
            
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("roles state sequence");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token, Token::Roles);
        assert_eq!(tokens[1].token, Token::State);
        assert_eq!(tokens[2].token, Token::Sequence);
        assert_eq!(tokens[3].token, Token::Eof);
    }
    
    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("Top Bottom Mount123 _private");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token, Token::Identifier("Top".to_string()));
        assert_eq!(tokens[1].token, Token::Identifier("Bottom".to_string()));
        assert_eq!(tokens[2].token, Token::Identifier("Mount123".to_string()));
        assert_eq!(tokens[3].token, Token::Identifier("_private".to_string()));
    }
    
    #[test]
    fn test_symbols() {
        let mut lexer = Lexer::new("{ } [ ] : -> ,");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token, Token::LeftBrace);
        assert_eq!(tokens[1].token, Token::RightBrace);
        assert_eq!(tokens[2].token, Token::LeftBracket);
        assert_eq!(tokens[3].token, Token::RightBracket);
        assert_eq!(tokens[4].token, Token::Colon);
        assert_eq!(tokens[5].token, Token::Arrow);
        assert_eq!(tokens[6].token, Token::Comma);
    }
    
    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("roles // this is a comment\nstate");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token, Token::Roles);
        assert_eq!(tokens[1].token, Token::State);
        assert_eq!(tokens[2].token, Token::Eof);
    }
    
    #[test]
    fn test_state_declaration() {
        let input = r#"
state Mount roles {
    Top, Bottom
}
"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token, Token::State);
        assert_eq!(tokens[1].token, Token::Identifier("Mount".to_string()));
        assert_eq!(tokens[2].token, Token::Roles);
        assert_eq!(tokens[3].token, Token::LeftBrace);
        assert_eq!(tokens[4].token, Token::Identifier("Top".to_string()));
        assert_eq!(tokens[5].token, Token::Comma);
        assert_eq!(tokens[6].token, Token::Identifier("Bottom".to_string()));
        assert_eq!(tokens[7].token, Token::RightBrace);
    }
    
    #[test]
    fn test_sequence_with_arrow() {
        let input = "sequence Test:\n    Action: State[Role] -> State2[Role2]";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token, Token::Sequence);
        assert_eq!(tokens[1].token, Token::Identifier("Test".to_string()));
        assert_eq!(tokens[2].token, Token::Colon);
        assert_eq!(tokens[3].token, Token::Identifier("Action".to_string()));
        assert_eq!(tokens[4].token, Token::Colon);
        assert_eq!(tokens[5].token, Token::Identifier("State".to_string()));
        assert_eq!(tokens[6].token, Token::LeftBracket);
        assert_eq!(tokens[7].token, Token::Identifier("Role".to_string()));
        assert_eq!(tokens[8].token, Token::RightBracket);
        assert_eq!(tokens[9].token, Token::Arrow);
    }

    #[test]
    fn test_group_declaration() {
        let input = "group GuardFamily { ClosedGuard, OpenGuard }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].token, Token::Group);
        assert_eq!(tokens[1].token, Token::Identifier("GuardFamily".to_string()));
        assert_eq!(tokens[2].token, Token::LeftBrace);
        assert_eq!(tokens[3].token, Token::Identifier("ClosedGuard".to_string()));
        assert_eq!(tokens[4].token, Token::Comma);
        assert_eq!(tokens[5].token, Token::Identifier("OpenGuard".to_string()));
        assert_eq!(tokens[6].token, Token::RightBrace);
    }
}
