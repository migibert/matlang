//! Parser for the Martial DSL
//!
//! Builds an Abstract Syntax Tree from a token stream.

use crate::ast::*;
use crate::lexer::{LexError, Position, PositionedToken, Token};
use std::fmt;

/// Parser error
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub position: Position,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parse error at {}: {}", self.position, self.message)
    }
}

impl From<LexError> for ParseError {
    fn from(err: LexError) -> Self {
        ParseError {
            message: err.message,
            position: err.position,
        }
    }
}

/// Parser for the Martial DSL
pub struct Parser {
    tokens: Vec<PositionedToken>,
    position: usize,
}

impl Parser {
    /// Create a new parser from a token stream
    pub fn new(tokens: Vec<PositionedToken>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    /// Get current position for error reporting
    fn current_position(&self) -> Position {
        if self.position < self.tokens.len() {
            self.tokens[self.position].position
        } else if !self.tokens.is_empty() {
            self.tokens[self.tokens.len() - 1].position
        } else {
            Position { line: 1, column: 1 }
        }
    }

    /// Peek at current token without consuming
    fn peek(&self) -> &Token {
        if self.position < self.tokens.len() {
            &self.tokens[self.position].token
        } else {
            &Token::Eof
        }
    }

    /// Consume and return current token
    fn advance(&mut self) -> &Token {
        if self.position < self.tokens.len() {
            let token = &self.tokens[self.position].token;
            self.position += 1;
            token
        } else {
            &Token::Eof
        }
    }

    /// Expect a specific token and consume it
    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let current = self.peek().clone();
        if current == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError {
                message: format!("Expected {}, got {}", expected, current),
                position: self.current_position(),
            })
        }
    }

    /// Expect an identifier and return it
    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        match self.peek().clone() {
            Token::Identifier(name) => {
                self.advance();
                Ok(name)
            }
            other => Err(ParseError {
                message: format!("Expected identifier, got {}", other),
                position: self.current_position(),
            }),
        }
    }

    /// Parse a complete martial file
    ///
    /// Grammar: program ::= declaration+
    pub fn parse(&mut self) -> Result<MartialFile, ParseError> {
        let mut declarations = Vec::new();

        while self.peek() != &Token::Eof {
            declarations.push(self.parse_declaration()?);
        }

        Ok(MartialFile { declarations })
    }

    /// Parse a declaration
    ///
    /// Grammar: declaration ::= roles_decl | state_decl | sequence_decl
    fn parse_declaration(&mut self) -> Result<Declaration, ParseError> {
        match self.peek() {
            Token::Roles => Ok(Declaration::Roles(self.parse_roles_decl()?)),
            Token::State => Ok(Declaration::State(self.parse_state_decl()?)),
            Token::Sequence => Ok(Declaration::Sequence(self.parse_sequence_decl()?)),
            other => Err(ParseError {
                message: format!(
                    "Expected declaration (roles, state, or sequence), got {}",
                    other
                ),
                position: self.current_position(),
            }),
        }
    }

    /// Parse a roles declaration
    ///
    /// Grammar: roles_decl ::= "roles" "{" IDENTIFIER { "," IDENTIFIER } "}"
    fn parse_roles_decl(&mut self) -> Result<RolesDecl, ParseError> {
        self.expect(Token::Roles)?;
        self.expect(Token::LeftBrace)?;

        let mut roles = Vec::new();
        roles.push(self.expect_identifier()?);

        while self.peek() == &Token::Comma {
            self.advance(); // consume comma
            roles.push(self.expect_identifier()?);
        }

        self.expect(Token::RightBrace)?;

        Ok(RolesDecl { roles })
    }

    /// Parse a state declaration
    ///
    /// Grammar: state_decl ::= "state" IDENTIFIER [ state_roles ]
    ///          state_roles ::= "roles" "{" IDENTIFIER { "," IDENTIFIER } "}"
    fn parse_state_decl(&mut self) -> Result<State, ParseError> {
        self.expect(Token::State)?;
        let name = self.expect_identifier()?;

        let allowed_roles = if self.peek() == &Token::Roles {
            self.advance(); // consume "roles"
            self.expect(Token::LeftBrace)?;

            let mut roles = Vec::new();
            roles.push(self.expect_identifier()?);

            while self.peek() == &Token::Comma {
                self.advance(); // consume comma
                roles.push(self.expect_identifier()?);
            }

            self.expect(Token::RightBrace)?;
            Some(roles)
        } else {
            None
        };

        Ok(State {
            name,
            allowed_roles,
        })
    }

    /// Parse a sequence declaration
    ///
    /// Grammar: sequence_decl ::= "sequence" IDENTIFIER ":" sequence_step+
    ///          sequence_step ::= IDENTIFIER ":" state_ref "->" state_ref
    fn parse_sequence_decl(&mut self) -> Result<Sequence, ParseError> {
        self.expect(Token::Sequence)?;
        let name = self.expect_identifier()?;
        self.expect(Token::Colon)?;

        let mut steps = Vec::new();

        // Parse at least one step
        steps.push(self.parse_sequence_step()?);

        // Parse additional steps
        // Keep parsing while we see identifiers (start of next step)
        while matches!(self.peek(), Token::Identifier(_)) {
            steps.push(self.parse_sequence_step()?);
        }

        Ok(Sequence { name, steps })
    }

    /// Parse a sequence step
    ///
    /// Grammar: sequence_step ::= IDENTIFIER ":" state_ref "->" state_ref
    fn parse_sequence_step(&mut self) -> Result<SequenceStep, ParseError> {
        let action_name = self.expect_identifier()?;
        self.expect(Token::Colon)?;
        let from = self.parse_state_ref()?;
        self.expect(Token::Arrow)?;
        let to = self.parse_state_ref()?;

        Ok(SequenceStep {
            action_name,
            from,
            to,
        })
    }

    /// Parse a state reference
    ///
    /// Grammar: state_ref ::= IDENTIFIER "[" IDENTIFIER "]"
    fn parse_state_ref(&mut self) -> Result<StateRef, ParseError> {
        let state = self.expect_identifier()?;
        self.expect(Token::LeftBracket)?;
        let role = self.expect_identifier()?;
        self.expect(Token::RightBracket)?;

        Ok(StateRef { state, role })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_input(input: &str) -> Result<MartialFile, ParseError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().map_err(ParseError::from)?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_parse_roles() {
        let input = "roles { Top, Bottom, Neutral }";
        let result = parse_input(input).unwrap();

        assert_eq!(result.declarations.len(), 1);
        match &result.declarations[0] {
            Declaration::Roles(roles_decl) => {
                assert_eq!(roles_decl.roles.len(), 3);
                assert_eq!(roles_decl.roles[0], "Top");
                assert_eq!(roles_decl.roles[1], "Bottom");
                assert_eq!(roles_decl.roles[2], "Neutral");
            }
            _ => panic!("Expected Roles declaration"),
        }
    }

    #[test]
    fn test_parse_state_simple() {
        let input = "state Standing";
        let result = parse_input(input).unwrap();

        assert_eq!(result.declarations.len(), 1);
        match &result.declarations[0] {
            Declaration::State(state) => {
                assert_eq!(state.name, "Standing");
                assert_eq!(state.allowed_roles, None);
            }
            _ => panic!("Expected State declaration"),
        }
    }

    #[test]
    fn test_parse_state_with_roles() {
        let input = "state Mount roles { Top, Bottom }";
        let result = parse_input(input).unwrap();

        assert_eq!(result.declarations.len(), 1);
        match &result.declarations[0] {
            Declaration::State(state) => {
                assert_eq!(state.name, "Mount");
                assert_eq!(state.allowed_roles, Some(vec!["Top".to_string(), "Bottom".to_string()]));
            }
            _ => panic!("Expected State declaration"),
        }
    }

    #[test]
    fn test_parse_sequence() {
        let input = r#"
sequence TestSequence:
    Action1: State1[Role1] -> State2[Role2]
    Action2: State2[Role2] -> State3[Role3]
"#;
        let result = parse_input(input).unwrap();

        assert_eq!(result.declarations.len(), 1);
        match &result.declarations[0] {
            Declaration::Sequence(seq) => {
                assert_eq!(seq.name, "TestSequence");
                assert_eq!(seq.steps.len(), 2);

                assert_eq!(seq.steps[0].action_name, "Action1");
                assert_eq!(seq.steps[0].from.state, "State1");
                assert_eq!(seq.steps[0].from.role, "Role1");
                assert_eq!(seq.steps[0].to.state, "State2");
                assert_eq!(seq.steps[0].to.role, "Role2");

                assert_eq!(seq.steps[1].action_name, "Action2");
                assert_eq!(seq.steps[1].from.state, "State2");
                assert_eq!(seq.steps[1].from.role, "Role2");
                assert_eq!(seq.steps[1].to.state, "State3");
                assert_eq!(seq.steps[1].to.role, "Role3");
            }
            _ => panic!("Expected Sequence declaration"),
        }
    }

    #[test]
    fn test_parse_multiple_declarations() {
        let input = r#"
roles { Top, Bottom }

state Mount roles { Top, Bottom }
state Standing

sequence MountEntry:
    Setup: Standing[Neutral] -> Mount[Top]
"#;
        let result = parse_input(input).unwrap();

        assert_eq!(result.declarations.len(), 4);
        assert!(matches!(result.declarations[0], Declaration::Roles(_)));
        assert!(matches!(result.declarations[1], Declaration::State(_)));
        assert!(matches!(result.declarations[2], Declaration::State(_)));
        assert!(matches!(result.declarations[3], Declaration::Sequence(_)));
    }

    #[test]
    fn test_parse_real_bjj_example() {
        let input = r#"
// BJJ example
roles {
    Top, Bottom, Neutral
}

state Standing
state Mount roles { Top, Bottom }

sequence GuardPass:
    Stack: ClosedGuard[Top] -> HalfGuard[Top]
    Pass: HalfGuard[Top] -> SideControl[Top]
"#;
        let result = parse_input(input).unwrap();
        assert_eq!(result.declarations.len(), 4);
    }
}
