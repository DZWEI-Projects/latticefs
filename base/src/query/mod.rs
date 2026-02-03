//! Lattice Query Language (LQL) module.
//!
//! This module provides the LQL query engine for LatticeFS:
//! - `lexer` - Token lexer for LQL
//! - `ast` - Abstract syntax tree types
//! - `parser` - Recursive descent parser
//! - `evaluator` - Query execution engine
//! - `explain` - Query explainability

pub mod ast;
pub mod evaluator;
pub mod explain;
pub mod lexer;
pub mod parser;

pub use ast::{
    CompareOp, Expr, MimePattern, ObjectRef, OrderBy, Predicate, Query, SortDirection, SortField,
    TimeField, TimeOp, TimeValue, TrustLevel,
};
pub use evaluator::QueryEvaluator;
pub use explain::{Explanation, Explainer, Reason};
pub use lexer::{Lexer, Token};
pub use parser::{parse, Parser};
