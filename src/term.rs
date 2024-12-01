//!
//! A term is some rule in our grammar
//!

use std::fmt;

/// Abstract syntax tree built from the BNF grammar
#[derive(Debug, Clone, PartialEq)]
pub enum Term<'src> {
  Variable(&'src str),
  Abstraction {
    param: &'src str,
    body: Box<Term<'src>>,
  },
  Application {
    lhs: Box<Term<'src>>,
    rhs: Box<Term<'src>>,
  },
}

/// Defines a way to transform some root term to its simplified version
pub trait Evaluate<'src> {
  fn evaluate(&mut self, term: &Term<'src>) -> Term<'src>;
}

impl<'src> fmt::Display for Term<'src> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Term::Variable(name) => write!(f, "{}", name),
      Term::Abstraction { param, body } => write!(f, "λ{}. {}", param, body),
      Term::Application { lhs, rhs } => {
        // insert parens for clarity in nested applications (show left-associative nature)
        let lhs_str = match **lhs {
          Term::Abstraction { .. } | Term::Application { .. } => format!("({})", lhs),
          _ => format!("{}", lhs),
        };
        let rhs_str = match **rhs {
          Term::Abstraction { .. } | Term::Application { .. } => format!("({})", rhs),
          _ => format!("{}", rhs),
        };
        write!(f, "{} {}", lhs_str, rhs_str)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod term {
    use super::*;

    #[test]
    fn display_variable() {
      let term = Term::Variable("x");
      assert_eq!(format!("{}", term), "x");
    }

    #[test]
    fn display_abstraction() {
      let term = Term::Abstraction {
        param: "x",
        body: Box::new(Term::Variable("x")),
      };
      assert_eq!(format!("{}", term), "λx. x");
    }

    #[test]
    fn display_application() {
      let term = Term::Application {
        lhs: Box::new(Term::Variable("x")),
        rhs: Box::new(Term::Variable("y")),
      };
      assert_eq!(format!("{}", term), "x y");
    }

    #[test]
    fn display_nested() {
      let term = Term::Application {
        lhs: Box::new(Term::Abstraction {
          param: "x",
          body: Box::new(Term::Variable("x")),
        }),
        rhs: Box::new(Term::Application {
          lhs: Box::new(Term::Variable("y")),
          rhs: Box::new(Term::Variable("z")),
        }),
      };
      assert_eq!(format!("{}", term), "(λx. x) (y z)");
    }
  }
}
