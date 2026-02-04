//! LQL Lexer (tokenizer).
//!
//! Converts LQL query strings into a stream of tokens for parsing.
//! Per LFS-002 section 2.

use crate::error::{LatticeError, Result};
use std::time::Duration;

/// Token types for LQL.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    And,
    Or,
    Not,
    Sort,
    Limit,
    Asc,
    Desc,

    // Predicate keywords
    Tag,
    Type,
    State,
    Trust,
    Updated,
    Created,
    Ref,
    References,
    Closure,

    // State values
    Draft,
    Review,
    Approved,
    Discarded,
    Sealed,
    Archived,

    // Trust levels
    Untrusted,
    Quarantined,
    Trusted,

    // Time operators
    Within,
    Before,
    After,
    Between,

    // Comparison operators
    Eq, // =
    Ne, // !=
    Gt, // >
    Lt, // <
    Ge, // >=
    Le, // <=

    // Punctuation
    Colon,  // :
    LParen, // (
    RParen, // )
    Slash,  // /
    Star,   // *

    // Literals
    Identifier(String),
    Number(i64),
    Duration(Duration),
    Timestamp(String),
    String(String),

    // End of input
    Eof,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::And => write!(f, "AND"),
            Token::Or => write!(f, "OR"),
            Token::Not => write!(f, "NOT"),
            Token::Sort => write!(f, "SORT"),
            Token::Limit => write!(f, "LIMIT"),
            Token::Asc => write!(f, "ASC"),
            Token::Desc => write!(f, "DESC"),
            Token::Tag => write!(f, "tag"),
            Token::Type => write!(f, "type"),
            Token::State => write!(f, "state"),
            Token::Trust => write!(f, "trust"),
            Token::Updated => write!(f, "updated"),
            Token::Created => write!(f, "created"),
            Token::Ref => write!(f, "ref"),
            Token::References => write!(f, "references"),
            Token::Closure => write!(f, "closure"),
            Token::Draft => write!(f, "draft"),
            Token::Review => write!(f, "review"),
            Token::Approved => write!(f, "approved"),
            Token::Discarded => write!(f, "discarded"),
            Token::Sealed => write!(f, "sealed"),
            Token::Archived => write!(f, "archived"),
            Token::Untrusted => write!(f, "untrusted"),
            Token::Quarantined => write!(f, "quarantined"),
            Token::Trusted => write!(f, "trusted"),
            Token::Within => write!(f, "within"),
            Token::Before => write!(f, "before"),
            Token::After => write!(f, "after"),
            Token::Between => write!(f, "between"),
            Token::Eq => write!(f, "="),
            Token::Ne => write!(f, "!="),
            Token::Gt => write!(f, ">"),
            Token::Lt => write!(f, "<"),
            Token::Ge => write!(f, ">="),
            Token::Le => write!(f, "<="),
            Token::Colon => write!(f, ":"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Slash => write!(f, "/"),
            Token::Star => write!(f, "*"),
            Token::Identifier(s) => write!(f, "{}", s),
            Token::Number(n) => write!(f, "{}", n),
            Token::Duration(d) => write!(f, "{}s", d.as_secs()),
            Token::Timestamp(s) => write!(f, "{}", s),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

/// LQL Lexer.
pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
    chars: Vec<char>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given input.
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            chars: input.chars().collect(),
        }
    }

    /// Get the current position in the input.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Peek at the current character without advancing.
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// Peek at the next character without advancing.
    // fn peek_next(&self) -> Option<char> {
    //     self.chars.get(self.pos + 1).copied()
    // }
    //
    /// Advance to the next character.
    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        Some(c)
    }

    /// Skip whitespace.
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else if c == '#' {
                // Skip comment until end of line
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    /// Read an identifier or keyword.
    fn read_identifier(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.pos].to_string()
    }

    /// Read a number.
    fn read_number(&mut self) -> i64 {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.pos].parse().unwrap_or(0)
    }

    /// Read a timestamp-like token starting at the current position.
    fn read_timestamp(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || matches!(c, '-' | 'T' | ':' | 'Z' | '+' | '.') {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.pos].to_string()
    }

    /// Read a string literal.
    fn read_string(&mut self) -> Result<String> {
        let start_pos = self.pos;
        self.advance(); // Skip opening quote

        let mut result = String::new();
        loop {
            match self.peek() {
                Some('"') => {
                    self.advance();
                    return Ok(result);
                }
                Some('\\') => {
                    self.advance();
                    match self.advance() {
                        Some('n') => result.push('\n'),
                        Some('t') => result.push('\t'),
                        Some('\\') => result.push('\\'),
                        Some('"') => result.push('"'),
                        Some(c) => result.push(c),
                        None => {
                            return Err(LatticeError::ParseError {
                                position: self.pos,
                                message: "Unterminated escape sequence".to_string(),
                            })
                        }
                    }
                }
                Some(c) => {
                    result.push(c);
                    self.advance();
                }
                None => {
                    return Err(LatticeError::ParseError {
                        position: start_pos,
                        message: "Unterminated string literal".to_string(),
                    })
                }
            }
        }
    }

    /// Get the next token.
    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        match self.peek() {
            None => Ok(Token::Eof),
            Some(c) => {
                // Single-character tokens
                match c {
                    ':' => {
                        self.advance();
                        Ok(Token::Colon)
                    }
                    '(' => {
                        self.advance();
                        Ok(Token::LParen)
                    }
                    ')' => {
                        self.advance();
                        Ok(Token::RParen)
                    }
                    '/' => {
                        self.advance();
                        Ok(Token::Slash)
                    }
                    '*' => {
                        self.advance();
                        Ok(Token::Star)
                    }
                    '=' => {
                        self.advance();
                        Ok(Token::Eq)
                    }
                    '!' => {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            Ok(Token::Ne)
                        } else {
                            Err(LatticeError::ParseError {
                                position: self.pos,
                                message: "Expected '=' after '!'".to_string(),
                            })
                        }
                    }
                    '>' => {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            Ok(Token::Ge)
                        } else {
                            Ok(Token::Gt)
                        }
                    }
                    '<' => {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            Ok(Token::Le)
                        } else {
                            Ok(Token::Lt)
                        }
                    }
                    '"' => {
                        let s = self.read_string()?;
                        Ok(Token::String(s))
                    }
                    _ if c.is_ascii_digit() => {
                        let num = self.read_number();

                        // If next char indicates timestamp, read full timestamp.
                        if matches!(self.peek(), Some('-' | 'T' | ':' | 'Z' | '+' | '.')) {
                            let mut timestamp = num.to_string();
                            timestamp.push_str(&self.read_timestamp());
                            return Ok(Token::Timestamp(timestamp));
                        }

                        // Check for duration suffix
                        if let Some(suffix) = self.peek() {
                            let duration = match suffix {
                                's' => {
                                    self.advance();
                                    Some(Duration::from_secs(num as u64))
                                }
                                'm' => {
                                    self.advance();
                                    Some(Duration::from_secs(num as u64 * 60))
                                }
                                'h' => {
                                    self.advance();
                                    Some(Duration::from_secs(num as u64 * 3600))
                                }
                                'd' => {
                                    self.advance();
                                    Some(Duration::from_secs(num as u64 * 86400))
                                }
                                'w' => {
                                    self.advance();
                                    Some(Duration::from_secs(num as u64 * 604800))
                                }
                                'y' => {
                                    self.advance();
                                    Some(Duration::from_secs(num as u64 * 31536000))
                                }
                                _ => None,
                            };

                            if let Some(d) = duration {
                                return Ok(Token::Duration(d));
                            }
                        }

                        Ok(Token::Number(num))
                    }
                    _ if c.is_alphabetic() || c == '_' => {
                        let ident = self.read_identifier();

                        // Check for keywords (case-insensitive)
                        let token = match ident.to_lowercase().as_str() {
                            "and" => Token::And,
                            "or" => Token::Or,
                            "not" => Token::Not,
                            "sort" => Token::Sort,
                            "limit" => Token::Limit,
                            "asc" => Token::Asc,
                            "desc" => Token::Desc,
                            "tag" => Token::Tag,
                            "type" => Token::Type,
                            "state" => Token::State,
                            "trust" => Token::Trust,
                            "updated" => Token::Updated,
                            "created" => Token::Created,
                            "ref" => Token::Ref,
                            "references" => Token::References,
                            "closure" => Token::Closure,
                            "draft" => Token::Draft,
                            "review" => Token::Review,
                            "approved" => Token::Approved,
                            "discarded" => Token::Discarded,
                            "sealed" => Token::Sealed,
                            "archived" => Token::Archived,
                            "untrusted" => Token::Untrusted,
                            "quarantined" => Token::Quarantined,
                            "trusted" => Token::Trusted,
                            "within" => Token::Within,
                            "before" => Token::Before,
                            "after" => Token::After,
                            "between" => Token::Between,
                            _ => Token::Identifier(ident),
                        };

                        Ok(token)
                    }
                    _ => Err(LatticeError::ParseError {
                        position: self.pos,
                        message: format!("Unexpected character: '{}'", c),
                    }),
                }
            }
        }
    }

    /// Peek at the next token without advancing.
    pub fn peek_token(&mut self) -> Result<Token> {
        let saved_pos = self.pos;
        let token = self.next_token()?;
        self.pos = saved_pos;
        Ok(token)
    }

    /// Tokenize the entire input.
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            if token == Token::Eof {
                break;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut lexer = Lexer::new("tag:project:phoenix");
        assert_eq!(lexer.next_token().unwrap(), Token::Tag);
        assert_eq!(lexer.next_token().unwrap(), Token::Colon);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("project".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Colon);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("phoenix".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_boolean_keywords() {
        let mut lexer = Lexer::new("AND OR NOT");
        assert_eq!(lexer.next_token().unwrap(), Token::And);
        assert_eq!(lexer.next_token().unwrap(), Token::Or);
        assert_eq!(lexer.next_token().unwrap(), Token::Not);
    }

    #[test]
    fn test_case_insensitive_keywords() {
        let mut lexer = Lexer::new("and AND And");
        assert_eq!(lexer.next_token().unwrap(), Token::And);
        assert_eq!(lexer.next_token().unwrap(), Token::And);
        assert_eq!(lexer.next_token().unwrap(), Token::And);
    }

    #[test]
    fn test_comparison_operators() {
        let mut lexer = Lexer::new("= != > < >= <=");
        assert_eq!(lexer.next_token().unwrap(), Token::Eq);
        assert_eq!(lexer.next_token().unwrap(), Token::Ne);
        assert_eq!(lexer.next_token().unwrap(), Token::Gt);
        assert_eq!(lexer.next_token().unwrap(), Token::Lt);
        assert_eq!(lexer.next_token().unwrap(), Token::Ge);
        assert_eq!(lexer.next_token().unwrap(), Token::Le);
    }

    #[test]
    fn test_duration() {
        let mut lexer = Lexer::new("7d 24h 30m 60s 2w 1y");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Duration(Duration::from_secs(7 * 86400))
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Duration(Duration::from_secs(24 * 3600))
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Duration(Duration::from_secs(30 * 60))
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Duration(Duration::from_secs(60))
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Duration(Duration::from_secs(2 * 604800))
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Duration(Duration::from_secs(31536000))
        );
    }

    #[test]
    fn test_number() {
        let mut lexer = Lexer::new("123 456");
        assert_eq!(lexer.next_token().unwrap(), Token::Number(123));
        assert_eq!(lexer.next_token().unwrap(), Token::Number(456));
    }

    #[test]
    fn test_timestamp() {
        let mut lexer = Lexer::new("2025-01-15 2025-01-15T10:30:00Z");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Timestamp("2025-01-15".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Timestamp("2025-01-15T10:30:00Z".to_string())
        );
    }

    #[test]
    fn test_string() {
        let mut lexer = Lexer::new("\"hello world\"");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("hello world".to_string())
        );
    }

    #[test]
    fn test_string_escapes() {
        let mut lexer = Lexer::new("\"hello\\nworld\\t!\"");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("hello\nworld\t!".to_string())
        );
    }

    #[test]
    fn test_mime_type() {
        let mut lexer = Lexer::new("type:application/pdf");
        assert_eq!(lexer.next_token().unwrap(), Token::Type);
        assert_eq!(lexer.next_token().unwrap(), Token::Colon);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("application".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Slash);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("pdf".to_string())
        );
    }

    #[test]
    fn test_wildcard_mime() {
        let mut lexer = Lexer::new("type:image/*");
        assert_eq!(lexer.next_token().unwrap(), Token::Type);
        assert_eq!(lexer.next_token().unwrap(), Token::Colon);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("image".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::Slash);
        assert_eq!(lexer.next_token().unwrap(), Token::Star);
    }

    #[test]
    fn test_comment() {
        let mut lexer = Lexer::new("tag:foo # this is a comment\ntag:bar");
        assert_eq!(lexer.next_token().unwrap(), Token::Tag);
        assert_eq!(lexer.next_token().unwrap(), Token::Colon);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("foo".to_string())
        );
        // Comment is skipped
        assert_eq!(lexer.next_token().unwrap(), Token::Tag);
    }

    #[test]
    fn test_complex_query() {
        let mut lexer = Lexer::new(
            "tag:project:phoenix AND type:application/pdf AND updated within 7d SORT updated DESC LIMIT 10",
        );
        let tokens = lexer.tokenize().unwrap();

        // tag, :, project, :, phoenix, AND, type, :, application, /, pdf, AND, updated, within, 7d, SORT, updated, DESC, LIMIT, 10
        assert_eq!(tokens.len(), 20);
        assert_eq!(tokens[0], Token::Tag);
        assert_eq!(tokens[5], Token::And);
        assert_eq!(tokens[11], Token::And);
    }

    #[test]
    fn test_parentheses() {
        let mut lexer = Lexer::new("(tag:a OR tag:b) AND tag:c");
        assert_eq!(lexer.next_token().unwrap(), Token::LParen);
        assert_eq!(lexer.next_token().unwrap(), Token::Tag);
        // ... skip to the closing paren
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.contains(&Token::RParen));
    }

    #[test]
    fn test_trust_predicate() {
        let mut lexer = Lexer::new("trust >= trusted");
        assert_eq!(lexer.next_token().unwrap(), Token::Trust);
        assert_eq!(lexer.next_token().unwrap(), Token::Ge);
        assert_eq!(lexer.next_token().unwrap(), Token::Trusted);
    }

    #[test]
    fn test_state_predicate() {
        let mut lexer = Lexer::new("state:approved");
        assert_eq!(lexer.next_token().unwrap(), Token::State);
        assert_eq!(lexer.next_token().unwrap(), Token::Colon);
        assert_eq!(lexer.next_token().unwrap(), Token::Approved);
    }

    #[test]
    fn test_unterminated_string() {
        let mut lexer = Lexer::new("\"unterminated");
        let result = lexer.next_token();
        assert!(result.is_err());
    }
}
