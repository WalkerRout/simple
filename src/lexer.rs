//!
//! Handles conversion from `&'src str` to `Iterator<Item=Token<'src>>`
//!

use std::iter::Peekable;
use std::str::CharIndices;

use crate::token::Token;

/// An iterator over lambda calculus tokens
pub struct Lexer<'src> {
  input: &'src str,
  chars: Peekable<CharIndices<'src>>,
}

impl<'src> Lexer<'src> {
  pub fn new(input: &'src str) -> Self {
    Self {
      input,
      chars: input.char_indices().peekable(),
    }
  }

  fn next_token(&mut self) -> Option<Token<'src>> {
    self.skip_whitespace();
    match self.peek_char() {
      Some('(') => {
        // consume '('
        self.chars.next();
        Some(Token::LParen)
      }
      Some(')') => {
        self.chars.next();
        Some(Token::RParen)
      }
      Some('λ') | Some('\\') => {
        self.chars.next();
        Some(Token::Lambda)
      }
      Some('.') => {
        self.chars.next();
        Some(Token::Dot)
      }
      Some(c) if c.is_ascii_lowercase() => self.read_binding(),
      None => None,
      // TODO:
      // introduce `Invalid` token...
      _ => panic!("Unexpected character: {:?}", self.chars.peek()),
    }
  }

  fn skip_whitespace(&mut self) {
    while let Some(ch) = self.peek_char() {
      if ch.is_whitespace() {
        self.chars.next();
      } else {
        break;
      }
    }
  }

  /// Read a lowercase identifier from the input
  /// - this method can fail if the input runs out
  fn read_binding(&mut self) -> Option<Token<'src>> {
    // where are we at right now?
    let start = self.chars.peek().map(|(idx, _)| *idx)?;

    while let Some(ch) = self.peek_char() {
      if ch.is_ascii_alphanumeric() {
        self.chars.next();
      } else {
        break;
      }
    }

    // where did we finish?
    let end = self
      .chars
      .peek()
      .map(|(idx, _)| *idx)
      .unwrap_or_else(|| self.input.len());

    // spit out the middle
    Some(Token::Binding(&self.input[start..end]))
  }

  fn peek_char(&mut self) -> Option<char> {
    self.chars.peek().map(|(_, c)| *c)
  }
}

impl<'a> Iterator for Lexer<'a> {
  type Item = Token<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    self.next_token()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod lexer {
    use super::*;

    #[test]
    fn next_token() {
      let mut lexer = Lexer::new("λx.x");
      assert_eq!(lexer.next_token(), Some(Token::Lambda));
      assert_eq!(lexer.next_token(), Some(Token::Binding("x")));
      assert_eq!(lexer.next_token(), Some(Token::Dot));
      assert_eq!(lexer.next_token(), Some(Token::Binding("x")));
      assert_eq!(lexer.next_token(), None);
    }

    #[test]
    fn next_token_parens() {
      let mut lexer = Lexer::new("(λx.(x x))");
      assert_eq!(lexer.next_token(), Some(Token::LParen));
      assert_eq!(lexer.next_token(), Some(Token::Lambda));
      assert_eq!(lexer.next_token(), Some(Token::Binding("x")));
      assert_eq!(lexer.next_token(), Some(Token::Dot));
      assert_eq!(lexer.next_token(), Some(Token::LParen));
      assert_eq!(lexer.next_token(), Some(Token::Binding("x")));
      assert_eq!(lexer.next_token(), Some(Token::Binding("x")));
      assert_eq!(lexer.next_token(), Some(Token::RParen));
      assert_eq!(lexer.next_token(), Some(Token::RParen));
      assert_eq!(lexer.next_token(), None);
    }

    #[test]
    fn next_token_whitespace() {
      let mut lexer = Lexer::new("   λ  x   . x  ");
      assert_eq!(lexer.next_token(), Some(Token::Lambda));
      assert_eq!(lexer.next_token(), Some(Token::Binding("x")));
      assert_eq!(lexer.next_token(), Some(Token::Dot));
      assert_eq!(lexer.next_token(), Some(Token::Binding("x")));
      assert_eq!(lexer.next_token(), None);
    }
  }
}
