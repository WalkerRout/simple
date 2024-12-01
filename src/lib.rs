//!
//! A simple lambda calculus interpreter, featuring token lexing, parsing, and
//! a syntax-tree walking interpreter
//!
//! Like all interpreters, we operate over some set of rules (grammar)...
//!
//! # Backus-Naur form Grammar
//!
//! ```text
//! term ::= appl
//!        | LAMBDA BIND DOT term
//!
//! appl ::= appl atom
//!        | atom
//!
//! atom ::= LPAREN term RPAREN
//!        | BIND
//! ```
//!
//! where our atoms are simply 'LAMBDA', 'BIND', 'DOT', 'LPAREN', and 'RPAREN'
//!

pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod term;
pub mod token;
