//! LQL Parser (recursive descent).
//!
//! Parses LQL query strings into an AST.
//! Per LFS-002 section 3, uses hand-written recursive descent.

use crate::error::{LatticeError, Result};
use crate::model::{ObjectID, State};
use crate::query::ast::*;
use crate::query::lexer::{Lexer, Token};
use time::format_description::well_known::Rfc3339;
use time::{Date, Month, OffsetDateTime, Time};

/// LQL Parser.
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given input.
    pub fn new(input: &'a str) -> Result<Self> {
        let mut lexer = Lexer::new(input);
        let current = lexer.next_token()?;
        Ok(Self { lexer, current })
    }

    /// Parse a complete query.
    pub fn parse(&mut self) -> Result<Query> {
        self.parse_query()
    }

    /// Get the current token.
    fn current(&self) -> &Token {
        &self.current
    }

    /// Advance to the next token.
    fn advance(&mut self) -> Result<Token> {
        let prev = std::mem::replace(&mut self.current, self.lexer.next_token()?);
        Ok(prev)
    }

    /// Check if the current token matches the expected token.
    fn check(&self, expected: &Token) -> bool {
        std::mem::discriminant(&self.current) == std::mem::discriminant(expected)
    }

    /// Consume the current token if it matches, return error otherwise.
    fn expect(&mut self, expected: Token) -> Result<Token> {
        if self.check(&expected) {
            self.advance()
        } else {
            Err(LatticeError::ParseError {
                position: self.lexer.position(),
                message: format!("Expected {:?}, got {:?}", expected, self.current),
            })
        }
    }

    /// Parse: query = expr order? limit?
    fn parse_query(&mut self) -> Result<Query> {
        let expr = self.parse_expr()?;

        let order = if self.check(&Token::Sort) {
            Some(self.parse_order()?)
        } else {
            None
        };

        let limit = if self.check(&Token::Limit) {
            Some(self.parse_limit()?)
        } else {
            None
        };

        Ok(Query { expr, order, limit })
    }

    /// Parse: expr = or_expr
    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_or()
    }

    /// Parse: or_expr = and_expr (OR and_expr)*
    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;

        while matches!(self.current(), Token::Or) {
            self.advance()?;
            let right = self.parse_and()?;
            left = Expr::or(left, right);
        }

        Ok(left)
    }

    /// Parse: and_expr = term (AND term)*
    fn parse_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_term()?;

        while matches!(self.current(), Token::And) {
            self.advance()?;
            let right = self.parse_term()?;
            left = Expr::and(left, right);
        }

        Ok(left)
    }

    /// Parse: term = predicate | NOT term | LPAREN expr RPAREN
    fn parse_term(&mut self) -> Result<Expr> {
        match self.current() {
            Token::Not => {
                self.advance()?;
                let term = self.parse_term()?;
                Ok(Expr::not(term))
            }
            Token::LParen => {
                self.advance()?;
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            _ => {
                let pred = self.parse_predicate()?;
                Ok(Expr::predicate(pred))
            }
        }
    }

    /// Parse a predicate.
    fn parse_predicate(&mut self) -> Result<Predicate> {
        match self.current() {
            Token::Tag => self.parse_tag_predicate(),
            Token::Type => self.parse_type_predicate(),
            Token::State => self.parse_state_predicate(),
            Token::Trust => self.parse_trust_predicate(),
            Token::Updated | Token::Created => self.parse_time_predicate(),
            Token::Ref => self.parse_ref_predicate(),
            Token::References => self.parse_references_predicate(),
            Token::Closure => self.parse_closure_predicate(),
            _ => Err(LatticeError::ParseError {
                position: self.lexer.position(),
                message: format!("Expected predicate, got {:?}", self.current),
            }),
        }
    }

    /// Parse: tag_pred = "tag" ":" tag_path
    fn parse_tag_predicate(&mut self) -> Result<Predicate> {
        self.expect(Token::Tag)?;
        self.expect(Token::Colon)?;

        let mut path = Vec::new();
        path.push(self.parse_tag_segment()?);

        while self.check(&Token::Colon) {
            self.advance()?;
            path.push(self.parse_tag_segment()?);
        }

        Ok(Predicate::Tag { path })
    }

    /// Parse: type_pred = "type" ":" mimetype
    fn parse_type_predicate(&mut self) -> Result<Predicate> {
        self.expect(Token::Type)?;
        self.expect(Token::Colon)?;

        // Support type:* (match all)
        let major = if self.check(&Token::Star) {
            self.advance()?;
            return Ok(Predicate::Type {
                mime: MimePattern {
                    major: "*".to_string(),
                    minor: None,
                },
            });
        } else {
            self.parse_identifier()?
        };

        if !self.check(&Token::Slash) {
            // Shorthand: type:pdf -> application/pdf
            return Ok(Predicate::Type {
                mime: MimePattern {
                    major: "application".to_string(),
                    minor: Some(major),
                },
            });
        }

        self.expect(Token::Slash)?;

        let minor = if self.check(&Token::Star) {
            self.advance()?;
            None
        } else {
            Some(self.parse_identifier()?)
        };

        Ok(Predicate::Type {
            mime: MimePattern { major, minor },
        })
    }

    /// Parse: state_pred = "state" ":" state_value
    fn parse_state_predicate(&mut self) -> Result<Predicate> {
        self.expect(Token::State)?;
        self.expect(Token::Colon)?;

        let state = match self.current() {
            Token::Draft => {
                self.advance()?;
                State::Draft
            }
            Token::Review => {
                self.advance()?;
                State::Review
            }
            Token::Approved => {
                self.advance()?;
                State::Approved
            }
            Token::Discarded => {
                self.advance()?;
                State::Discarded
            }
            Token::Sealed => {
                self.advance()?;
                State::Sealed
            }
            Token::Archived => {
                self.advance()?;
                State::Archived
            }
            _ => {
                return Err(LatticeError::ParseError {
                    position: self.lexer.position(),
                    message: format!(
                        "Expected state value (draft|review|approved|discarded|sealed|archived), got {:?}",
                        self.current
                    ),
                })
            }
        };

        Ok(Predicate::State { state })
    }

    /// Parse: trust_pred = "trust" op trust_level
    fn parse_trust_predicate(&mut self) -> Result<Predicate> {
        self.expect(Token::Trust)?;

        let op = self.parse_compare_op()?;

        let level = match self.current() {
            Token::Untrusted => {
                self.advance()?;
                TrustLevel::Untrusted
            }
            Token::Quarantined => {
                self.advance()?;
                TrustLevel::Quarantined
            }
            Token::Trusted => {
                self.advance()?;
                TrustLevel::Trusted
            }
            Token::Approved => {
                self.advance()?;
                TrustLevel::Approved
            }
            Token::Number(n) => {
                let n = *n;
                self.advance()?;
                TrustLevel::Numeric(n as u8)
            }
            Token::Identifier(s) if s.to_lowercase() == "medium" => {
                self.advance()?;
                TrustLevel::Numeric(50)
            }
            _ => {
                return Err(LatticeError::ParseError {
                    position: self.lexer.position(),
                    message: format!("Expected trust level, got {:?}", self.current),
                })
            }
        };

        Ok(Predicate::Trust { op, level })
    }

    /// Parse: time_pred = time_field time_op time_value
    fn parse_time_predicate(&mut self) -> Result<Predicate> {
        let field = match self.current() {
            Token::Updated => {
                self.advance()?;
                TimeField::Updated
            }
            Token::Created => {
                self.advance()?;
                TimeField::Created
            }
            _ => {
                return Err(LatticeError::ParseError {
                    position: self.lexer.position(),
                    message: format!(
                        "Expected time field (updated|created), got {:?}",
                        self.current
                    ),
                })
            }
        };

        let op = match self.current() {
            Token::Within => {
                self.advance()?;
                TimeOp::Within
            }
            Token::Before => {
                self.advance()?;
                TimeOp::Before
            }
            Token::After => {
                self.advance()?;
                TimeOp::After
            }
            Token::Between => {
                self.advance()?;
                TimeOp::Between
            }
            _ => {
                return Err(LatticeError::ParseError {
                    position: self.lexer.position(),
                    message: format!(
                        "Expected time operator (within|before|after|between), got {:?}",
                        self.current
                    ),
                })
            }
        };

        let value = match op {
            TimeOp::Within => {
                let duration = match self.current() {
                    Token::Duration(d) => {
                        let d = *d;
                        self.advance()?;
                        d
                    }
                    _ => {
                        return Err(LatticeError::ParseError {
                            position: self.lexer.position(),
                            message: format!(
                                "Expected duration after 'within', got {:?}",
                                self.current
                            ),
                        })
                    }
                };
                TimeValue::Duration(duration)
            }
            TimeOp::Before | TimeOp::After => {
                let timestamp = self.parse_timestamp_value()?;
                TimeValue::Timestamp(timestamp)
            }
            TimeOp::Between => {
                let start = self.parse_timestamp_value()?;
                let end = self.parse_timestamp_value()?;
                TimeValue::Range { start, end }
            }
        };

        Ok(Predicate::Time { field, op, value })
    }

    /// Parse: ref_pred = "ref" ":" ref
    fn parse_ref_predicate(&mut self) -> Result<Predicate> {
        self.expect(Token::Ref)?;
        self.expect(Token::Colon)?;

        let reference = self.parse_object_ref()?;

        Ok(Predicate::Ref { reference })
    }

    /// Parse: references_pred = "references" "(" ref ")"
    fn parse_references_predicate(&mut self) -> Result<Predicate> {
        self.expect(Token::References)?;
        self.expect(Token::LParen)?;

        let target = self.parse_object_ref()?;

        self.expect(Token::RParen)?;

        Ok(Predicate::References { target })
    }

    /// Parse: closure_pred = "closure" "(" ref ")"
    fn parse_closure_predicate(&mut self) -> Result<Predicate> {
        self.expect(Token::Closure)?;
        self.expect(Token::LParen)?;

        let root = self.parse_object_ref()?;

        self.expect(Token::RParen)?;

        Ok(Predicate::Closure { root })
    }

    /// Parse an object reference.
    fn parse_object_ref(&mut self) -> Result<ObjectRef> {
        match self.current() {
            Token::Ref => {
                // Allow references(ref:...) style
                self.advance()?;
                self.expect(Token::Colon)?;
                self.parse_object_ref()
            }
            Token::Identifier(s) => {
                // Could be UUID or hash
                let s = s.clone();
                self.advance()?;

                // Try parsing as UUID
                if let Ok(uuid) = uuid::Uuid::parse_str(&s) {
                    return Ok(ObjectRef::Id(ObjectID::from_uuid(uuid)));
                }

                // Otherwise treat as hash
                Ok(ObjectRef::Hash(s))
            }
            Token::Tag => {
                // Tag reference: tag:project:phoenix
                self.advance()?;
                self.expect(Token::Colon)?;

                let mut path = Vec::new();
                path.push(self.parse_tag_segment()?);

                while self.check(&Token::Colon) {
                    self.advance()?;
                    path.push(self.parse_tag_segment()?);
                }

                Ok(ObjectRef::Tag(path))
            }
            Token::String(s) => {
                let s = s.clone();
                self.advance()?;
                Ok(ObjectRef::Alias(s))
            }
            _ => Err(LatticeError::ParseError {
                position: self.lexer.position(),
                message: format!("Expected object reference, got {:?}", self.current),
            }),
        }
    }

    /// Parse a comparison operator.
    fn parse_compare_op(&mut self) -> Result<CompareOp> {
        let op = match self.current() {
            Token::Eq => CompareOp::Eq,
            Token::Ne => CompareOp::Ne,
            Token::Gt => CompareOp::Gt,
            Token::Lt => CompareOp::Lt,
            Token::Ge => CompareOp::Ge,
            Token::Le => CompareOp::Le,
            _ => {
                return Err(LatticeError::ParseError {
                    position: self.lexer.position(),
                    message: format!("Expected comparison operator, got {:?}", self.current),
                })
            }
        };
        self.advance()?;
        Ok(op)
    }

    /// Parse: order = "SORT" field direction?
    fn parse_order(&mut self) -> Result<OrderBy> {
        self.expect(Token::Sort)?;

        let field = match self.current() {
            Token::Updated => {
                self.advance()?;
                SortField::Updated
            }
            Token::Created => {
                self.advance()?;
                SortField::Created
            }
            Token::Identifier(s) if s.to_lowercase() == "size" => {
                self.advance()?;
                SortField::Size
            }
            Token::Trust => {
                self.advance()?;
                SortField::Trust
            }
            _ => {
                return Err(LatticeError::ParseError {
                    position: self.lexer.position(),
                    message: format!("Expected sort field, got {:?}", self.current),
                })
            }
        };

        let direction = match self.current() {
            Token::Asc => {
                self.advance()?;
                SortDirection::Asc
            }
            Token::Desc => {
                self.advance()?;
                SortDirection::Desc
            }
            _ => SortDirection::Desc, // Default
        };

        Ok(OrderBy { field, direction })
    }

    /// Parse: limit = "LIMIT" number
    fn parse_limit(&mut self) -> Result<usize> {
        self.expect(Token::Limit)?;

        match self.current() {
            Token::Number(n) => {
                let n = *n as usize;
                self.advance()?;
                Ok(n)
            }
            _ => Err(LatticeError::ParseError {
                position: self.lexer.position(),
                message: format!("Expected number after LIMIT, got {:?}", self.current),
            }),
        }
    }

    /// Parse an identifier.
    fn parse_identifier(&mut self) -> Result<String> {
        match &self.current {
            Token::Identifier(s) => {
                let s = s.clone();
                self.advance()?;
                Ok(s)
            }
            // Also accept some keywords as identifiers in certain contexts
            Token::Draft => {
                self.advance()?;
                Ok("draft".to_string())
            }
            Token::Review => {
                self.advance()?;
                Ok("review".to_string())
            }
            Token::Approved => {
                self.advance()?;
                Ok("approved".to_string())
            }
            _ => Err(LatticeError::ParseError {
                position: self.lexer.position(),
                message: format!("Expected identifier, got {:?}", self.current),
            }),
        }
    }

    /// Parse a tag path segment, allowing quoted strings, wildcards, and slash-containing values.
    fn parse_tag_segment(&mut self) -> Result<String> {
        let mut segment = self.parse_tag_atom()?;

        while self.check(&Token::Slash) {
            self.advance()?;
            let next = self.parse_tag_atom()?;
            segment.push('/');
            segment.push_str(&next);
        }

        Ok(segment)
    }

    fn parse_tag_atom(&mut self) -> Result<String> {
        match &self.current {
            Token::String(s) => {
                let s = s.clone();
                self.advance()?;
                Ok(s)
            }
            Token::Star => {
                self.advance()?;
                Ok("*".to_string())
            }
            _ => self.parse_identifier(),
        }
    }

    /// Parse a timestamp value and return Unix microseconds.
    fn parse_timestamp_value(&mut self) -> Result<i64> {
        let raw = match self.current() {
            Token::Timestamp(s) => {
                let s = s.clone();
                self.advance()?;
                s
            }
            Token::String(s) => {
                let s = s.clone();
                self.advance()?;
                s
            }
            Token::Number(n) => {
                let n = *n;
                self.advance()?;
                // Treat bare numbers as epoch seconds
                return Ok(n * 1_000_000);
            }
            _ => {
                return Err(LatticeError::ParseError {
                    position: self.lexer.position(),
                    message: format!("Expected timestamp, got {:?}", self.current),
                })
            }
        };

        parse_timestamp_to_micros(&raw).map_err(|e| LatticeError::ParseError {
            position: self.lexer.position(),
            message: format!("Invalid timestamp '{}': {}", raw, e),
        })
    }
}

fn parse_timestamp_to_micros(raw: &str) -> Result<i64> {
    // RFC3339 / ISO 8601
    if raw.contains('T') || raw.ends_with('Z') || raw.contains('+') || raw.contains(':') {
        let dt = OffsetDateTime::parse(raw, &Rfc3339)
            .map_err(|e| LatticeError::Serialization(format!("RFC3339 parse error: {}", e)))?;
        return Ok(dt.unix_timestamp() * 1_000_000 + i64::from(dt.nanosecond() / 1000));
    }

    // YYYY-MM-DD
    if raw.len() == 10
        && raw.as_bytes().get(4) == Some(&b'-')
        && raw.as_bytes().get(7) == Some(&b'-')
    {
        let year: i32 = raw[0..4]
            .parse()
            .map_err(|_| LatticeError::Serialization("Invalid year in date".to_string()))?;
        let month: u8 = raw[5..7]
            .parse()
            .map_err(|_| LatticeError::Serialization("Invalid month in date".to_string()))?;
        let day: u8 = raw[8..10]
            .parse()
            .map_err(|_| LatticeError::Serialization("Invalid day in date".to_string()))?;

        let date = Date::from_calendar_date(
            year,
            Month::try_from(month)
                .map_err(|_| LatticeError::Serialization("Invalid month in date".to_string()))?,
            day,
        )
        .map_err(|e| LatticeError::Serialization(format!("Invalid date: {}", e)))?;

        let dt = date.with_time(Time::MIDNIGHT).assume_utc();
        return Ok(dt.unix_timestamp() * 1_000_000);
    }

    // YYYY-MM (start of month)
    if raw.len() == 7 && raw.as_bytes().get(4) == Some(&b'-') {
        let year: i32 = raw[0..4]
            .parse()
            .map_err(|_| LatticeError::Serialization("Invalid year in date".to_string()))?;
        let month: u8 = raw[5..7]
            .parse()
            .map_err(|_| LatticeError::Serialization("Invalid month in date".to_string()))?;

        let date = Date::from_calendar_date(
            year,
            Month::try_from(month)
                .map_err(|_| LatticeError::Serialization("Invalid month in date".to_string()))?,
            1,
        )
        .map_err(|e| LatticeError::Serialization(format!("Invalid date: {}", e)))?;

        let dt = date.with_time(Time::MIDNIGHT).assume_utc();
        return Ok(dt.unix_timestamp() * 1_000_000);
    }

    Err(LatticeError::Serialization(
        "Unsupported timestamp format".to_string(),
    ))
}

/// Parse an LQL query string.
pub fn parse(input: &str) -> Result<Query> {
    let mut parser = Parser::new(input)?;
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_simple_tag() {
        let query = parse("tag:project:phoenix").unwrap();

        match query.expr {
            Expr::Predicate(Predicate::Tag { path }) => {
                assert_eq!(path, vec!["project", "phoenix"]);
            }
            _ => panic!("Expected tag predicate"),
        }
    }

    #[test]
    fn test_tag_with_slash_value() {
        let query = parse("tag:auto:mimetype:image/jpeg").unwrap();

        match query.expr {
            Expr::Predicate(Predicate::Tag { path }) => {
                assert_eq!(path, vec!["auto", "mimetype", "image/jpeg"]);
            }
            _ => panic!("Expected tag predicate"),
        }
    }

    #[test]
    fn test_tag_with_wildcard_value() {
        let query = parse("tag:auto:mimetype:image/*").unwrap();

        match query.expr {
            Expr::Predicate(Predicate::Tag { path }) => {
                assert_eq!(path, vec!["auto", "mimetype", "image/*"]);
            }
            _ => panic!("Expected tag predicate"),
        }
    }

    #[test]
    fn test_tag_with_wildcard_major_minor() {
        let query = parse("tag:auto:mimetype:*/*").unwrap();

        match query.expr {
            Expr::Predicate(Predicate::Tag { path }) => {
                assert_eq!(path, vec!["auto", "mimetype", "*/*"]);
            }
            _ => panic!("Expected tag predicate"),
        }
    }

    #[test]
    fn test_type_predicate() {
        let query = parse("type:application/pdf").unwrap();

        match query.expr {
            Expr::Predicate(Predicate::Type { mime }) => {
                assert_eq!(mime.major, "application");
                assert_eq!(mime.minor, Some("pdf".to_string()));
            }
            _ => panic!("Expected type predicate"),
        }
    }

    #[test]
    fn test_type_wildcard() {
        let query = parse("type:image/*").unwrap();

        match query.expr {
            Expr::Predicate(Predicate::Type { mime }) => {
                assert_eq!(mime.major, "image");
                assert_eq!(mime.minor, None);
            }
            _ => panic!("Expected type predicate"),
        }
    }

    #[test]
    fn test_state_predicate() {
        let query = parse("state:approved").unwrap();

        match query.expr {
            Expr::Predicate(Predicate::State { state }) => {
                assert_eq!(state, State::Approved);
            }
            _ => panic!("Expected state predicate"),
        }
    }

    #[test]
    fn test_trust_predicate() {
        let query = parse("trust >= trusted").unwrap();

        match query.expr {
            Expr::Predicate(Predicate::Trust { op, level }) => {
                assert_eq!(op, CompareOp::Ge);
                assert_eq!(level, TrustLevel::Trusted);
            }
            _ => panic!("Expected trust predicate"),
        }
    }

    #[test]
    fn test_time_predicate() {
        let query = parse("updated within 7d").unwrap();

        match query.expr {
            Expr::Predicate(Predicate::Time { field, op, value }) => {
                assert_eq!(field, TimeField::Updated);
                assert_eq!(op, TimeOp::Within);
                assert_eq!(value, TimeValue::Duration(Duration::from_secs(7 * 86400)));
            }
            _ => panic!("Expected time predicate"),
        }
    }

    #[test]
    fn test_and_expression() {
        let query = parse("tag:project AND type:application/pdf").unwrap();

        match query.expr {
            Expr::And(left, right) => {
                assert!(matches!(*left, Expr::Predicate(Predicate::Tag { .. })));
                assert!(matches!(*right, Expr::Predicate(Predicate::Type { .. })));
            }
            _ => panic!("Expected AND expression"),
        }
    }

    #[test]
    fn test_or_expression() {
        let query = parse("tag:a OR tag:b").unwrap();

        match query.expr {
            Expr::Or(_, _) => {}
            _ => panic!("Expected OR expression"),
        }
    }

    #[test]
    fn test_not_expression() {
        let query = parse("NOT state:archived").unwrap();

        match query.expr {
            Expr::Not(inner) => {
                assert!(matches!(*inner, Expr::Predicate(Predicate::State { .. })));
            }
            _ => panic!("Expected NOT expression"),
        }
    }

    #[test]
    fn test_parentheses() {
        let query = parse("(tag:a OR tag:b) AND tag:c").unwrap();

        match query.expr {
            Expr::And(left, _) => {
                assert!(matches!(*left, Expr::Or(_, _)));
            }
            _ => panic!("Expected AND with OR on left"),
        }
    }

    #[test]
    fn test_sort() {
        let query = parse("tag:project SORT updated DESC").unwrap();

        assert!(query.order.is_some());
        let order = query.order.unwrap();
        assert_eq!(order.field, SortField::Updated);
        assert_eq!(order.direction, SortDirection::Desc);
    }

    #[test]
    fn test_limit() {
        let query = parse("tag:project LIMIT 10").unwrap();

        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_full_query() {
        let query =
            parse("tag:project:phoenix AND type:application/pdf SORT updated DESC LIMIT 10")
                .unwrap();

        assert!(matches!(query.expr, Expr::And(_, _)));
        assert!(query.order.is_some());
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_complex_boolean() {
        let query =
            parse("(tag:urgent OR tag:priority:high) AND state:review AND NOT state:archived")
                .unwrap();

        // The structure should be: AND(AND(OR(...), state:review), NOT(state:archived))
        match query.expr {
            Expr::And(_, _) => {}
            _ => panic!("Expected AND expression at top level"),
        }
    }

    #[test]
    fn test_references_predicate() {
        let query = parse("references(tag:project:phoenix)").unwrap();

        match query.expr {
            Expr::Predicate(Predicate::References { target }) => {
                assert!(matches!(target, ObjectRef::Tag(_)));
            }
            _ => panic!("Expected references predicate"),
        }
    }

    #[test]
    fn test_closure_predicate() {
        let query = parse("closure(tag:project)").unwrap();

        match query.expr {
            Expr::Predicate(Predicate::Closure { root }) => {
                assert!(matches!(root, ObjectRef::Tag(_)));
            }
            _ => panic!("Expected closure predicate"),
        }
    }

    #[test]
    fn test_ref_alias() {
        let query = parse("ref:\"project-readme\"").unwrap();

        match query.expr {
            Expr::Predicate(Predicate::Ref { reference }) => {
                assert!(matches!(reference, ObjectRef::Alias(_)));
            }
            _ => panic!("Expected ref predicate"),
        }
    }

    #[test]
    fn test_parse_error() {
        let result = parse("tag:");
        assert!(result.is_err());
    }

    #[test]
    fn test_trust_numeric() {
        let query = parse("trust >= 50").unwrap();

        match query.expr {
            Expr::Predicate(Predicate::Trust { op, level }) => {
                assert_eq!(op, CompareOp::Ge);
                assert_eq!(level, TrustLevel::Numeric(50));
            }
            _ => panic!("Expected trust predicate"),
        }
    }

    #[test]
    fn test_operator_precedence() {
        let query = parse("tag:a OR tag:b AND tag:c").unwrap();

        match query.expr {
            Expr::Or(left, right) => {
                assert!(matches!(*left, Expr::Predicate(Predicate::Tag { .. })));
                assert!(matches!(*right, Expr::And(_, _)));
            }
            _ => panic!("Expected OR with AND on right due to precedence"),
        }
    }

    #[test]
    fn test_time_between() {
        let query = parse("updated between 2025-01-01 2025-02-01").unwrap();

        match query.expr {
            Expr::Predicate(Predicate::Time { op, value, .. }) => {
                assert_eq!(op, TimeOp::Between);
                match value {
                    TimeValue::Range { start, end } => {
                        assert!(start < end);
                    }
                    _ => panic!("Expected range value"),
                }
            }
            _ => panic!("Expected time predicate"),
        }
    }

    #[test]
    fn test_type_any() {
        let query = parse("type:*").unwrap();

        match query.expr {
            Expr::Predicate(Predicate::Type { mime }) => {
                assert_eq!(mime.major, "*");
                assert_eq!(mime.minor, None);
            }
            _ => panic!("Expected type predicate"),
        }
    }

    #[test]
    fn test_type_shorthand_pdf() {
        let query = parse("type:pdf").unwrap();

        match query.expr {
            Expr::Predicate(Predicate::Type { mime }) => {
                assert_eq!(mime.major, "application");
                assert_eq!(mime.minor, Some("pdf".to_string()));
            }
            _ => panic!("Expected type predicate"),
        }
    }

    #[test]
    fn test_trust_medium() {
        let query = parse("trust >= medium").unwrap();

        match query.expr {
            Expr::Predicate(Predicate::Trust { op, level }) => {
                assert_eq!(op, CompareOp::Ge);
                assert_eq!(level, TrustLevel::Numeric(50));
            }
            _ => panic!("Expected trust predicate"),
        }
    }
}
