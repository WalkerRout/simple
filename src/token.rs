//!
//! Simplest available unit of the language, used to represent atoms in the grammar
//!

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token<'src> {
  LParen,             // '('
  RParen,             // ')'
  Lambda,             // 'λ' or '\'
  Dot,                // '.'
  Binding(&'src str), // some lowercase id
}
