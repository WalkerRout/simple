//!
//! Handles conversion from `Iterator<Item=Token<'src>>` to `Term<'src>`
//!

use crate::term::Term;
use crate::token::Token;

/// Our parser can fail, so we must have some way to represent failure
#[derive(Debug, PartialEq)]
pub enum ParseError<'src> {
  /// We ran out of input while in the middle of parsing something
  UnexpectedEof,
  /// We ran into a token we didn't expect to see
  UnexpectedToken(Token<'src>),
}

/// Explicit return type for functions that do parsing, to distinguish them
pub type ParseResult<'src> = Result<Term<'src>, ParseError<'src>>;

/// Process all tokens in provided iterator
pub struct Parser<'src, I> {
  tokens: I,
  current_token: Option<Token<'src>>,
}

impl<'src, I> Parser<'src, I>
where
  I: Iterator<Item = Token<'src>>,
{
  pub fn new(mut tokens: I) -> Self {
    let current_token = tokens.next();
    Self {
      tokens,
      current_token,
    }
  }

  /// Try to convert the parser's provided iterator into some `Term<'src>`
  pub fn parse(&mut self) -> ParseResult<'src> {
    let expr = self.parse_application()?;
    let () = self.eof()?;
    Ok(expr)
  }

  fn parse_abstraction(&mut self) -> ParseResult<'src> {
    let () = self.eat(Token::Lambda)?;
    let param = self.eat_binding()?;
    let () = self.eat(Token::Dot)?;
    let body = self.parse_application()?;
    Ok(Term::Abstraction {
      param,
      body: Box::new(body),
    })
  }

  fn parse_parenthesized(&mut self) -> ParseResult<'src> {
    let () = self.eat(Token::LParen)?;
    let term = self.parse_application()?;
    let () = self.eat(Token::RParen)?;
    Ok(term)
  }

  fn parse_atom(&mut self) -> ParseResult<'src> {
    match self.peek() {
      Some(Token::Binding(_)) => Ok(Term::Variable(self.eat_binding()?)),
      Some(Token::LParen) => self.parse_parenthesized(),
      Some(Token::Lambda) => self.parse_abstraction(),
      Some(tok) => Err(ParseError::UnexpectedToken(tok.clone())),
      None => Err(ParseError::UnexpectedEof),
    }
  }

  fn parse_application(&mut self) -> ParseResult<'src> {
    let mut term = self.parse_atom()?;
    // keep parsing while the next token can start an atom
    while matches!(self.peek(), Some(Token::Binding(_)) | Some(Token::LParen)) {
      let rhs = self.parse_atom()?;
      term = Term::Application {
        lhs: Box::new(term),
        rhs: Box::new(rhs),
      };
    }
    Ok(term)
  }

  fn eof(&mut self) -> Result<(), ParseError<'src>> {
    if let Some(tok) = self.next() {
      Err(ParseError::UnexpectedToken(tok))
    } else {
      Ok(())
    }
  }

  fn peek(&self) -> Option<&Token<'src>> {
    self.current_token.as_ref()
  }

  fn next(&mut self) -> Option<Token<'src>> {
    let token = self.current_token.clone();
    self.current_token = self.tokens.next();
    token
  }

  fn next_eof(&mut self) -> Result<Token<'src>, ParseError<'src>> {
    self.next().ok_or(ParseError::UnexpectedEof)
  }

  fn eat(&mut self, expected: Token<'src>) -> Result<(), ParseError<'src>> {
    let actual = self.next_eof()?;
    if actual == expected {
      Ok(())
    } else {
      Err(ParseError::UnexpectedToken(actual))
    }
  }

  fn eat_binding(&mut self) -> Result<&'src str, ParseError<'src>> {
    match self.next_eof()? {
      Token::Binding(name) => Ok(name),
      other => Err(ParseError::UnexpectedToken(other)),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod parser {
    use super::*;

    use crate::lexer::Lexer;

    #[test]
    fn parse_variable_single_variable() {
      let input = "x";
      let lexer = Lexer::new(input);
      let mut parser = Parser::new(lexer);

      let ast = parser.parse();
      assert_eq!(ast, Ok(Term::Variable("x")));
    }

    #[test]
    fn parse_abstraction_simple_lambda() {
      let input = "\\x.x";
      let lexer = Lexer::new(input);
      let mut parser = Parser::new(lexer);

      let ast = parser.parse();
      assert_eq!(
        ast,
        Ok(Term::Abstraction {
          param: "x",
          body: Box::new(Term::Variable("x"))
        })
      );
    }

    #[test]
    fn parse_abstraction_nested_lambda() {
      let input = "\\x.\\y.x";
      let lexer = Lexer::new(input);
      let mut parser = Parser::new(lexer);

      let ast = parser.parse();
      assert_eq!(
        ast,
        Ok(Term::Abstraction {
          param: "x",
          body: Box::new(Term::Abstraction {
            param: "y",
            body: Box::new(Term::Variable("x"))
          })
        })
      );
    }

    #[test]
    fn parse_application_simple_application() {
      let input = "x y";
      let lexer = Lexer::new(input);
      let mut parser = Parser::new(lexer);

      let ast = parser.parse();
      assert_eq!(
        ast,
        Ok(Term::Application {
          lhs: Box::new(Term::Variable("x")),
          rhs: Box::new(Term::Variable("y"))
        })
      );
    }

    #[test]
    fn parse_application_nested_application() {
      let input = "x y z";
      let lexer = Lexer::new(input);
      let mut parser = Parser::new(lexer);

      let ast = parser.parse();
      assert_eq!(
        ast,
        Ok(Term::Application {
          lhs: Box::new(Term::Application {
            lhs: Box::new(Term::Variable("x")),
            rhs: Box::new(Term::Variable("y")),
          }),
          rhs: Box::new(Term::Variable("z"))
        })
      );
    }

    #[test]
    fn parse_combination_lambda_and_application() {
      let input = "(\\x.x) y";
      let lexer = Lexer::new(input);
      let mut parser = Parser::new(lexer);

      let ast = parser.parse();
      assert_eq!(
        ast,
        Ok(Term::Application {
          lhs: Box::new(Term::Abstraction {
            param: "x",
            body: Box::new(Term::Variable("x"))
          }),
          rhs: Box::new(Term::Variable("y"))
        })
      );
    }

    #[test]
    fn parse_nested_parentheses_with_application() {
      let input = "(x (y z))";
      let lexer = Lexer::new(input);
      let mut parser = Parser::new(lexer);

      let ast = parser.parse();
      assert_eq!(
        ast,
        Ok(Term::Application {
          lhs: Box::new(Term::Variable("x")),
          rhs: Box::new(Term::Application {
            lhs: Box::new(Term::Variable("y")),
            rhs: Box::new(Term::Variable("z"))
          })
        })
      );
    }

    #[test]
    fn parse_complex_expression_with_nested_lambda() {
      let input = "\\x.(x (\\y.y))";
      let lexer = Lexer::new(input);
      let mut parser = Parser::new(lexer);

      let ast = parser.parse();
      assert_eq!(
        ast,
        Ok(Term::Abstraction {
          param: "x",
          body: Box::new(Term::Application {
            lhs: Box::new(Term::Variable("x")),
            rhs: Box::new(Term::Abstraction {
              param: "y",
              body: Box::new(Term::Variable("y"))
            })
          })
        })
      );
    }
  }
}
