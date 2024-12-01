//!
//! Provide some concrete way to simplify/evaluate a root `Term<'src>` node
//!

use crate::term::{Evaluate, Term};

/// Exists only as an implementor of `term::Evaluate`
pub struct Interpreter;

impl<'src> Interpreter {
  /// Recursively simplify a given term to evaluate it
  fn evaluate_term(&mut self, term: &Term<'src>) -> Term<'src> {
    match term {
      // Evaluate applications
      Term::Application { lhs, rhs } => {
        let lhs_eval = self.evaluate_term(lhs);
        let rhs_eval = self.evaluate_term(rhs);
        // Apply the abstraction if the left-hand side is one
        if let Term::Abstraction { param, body } = lhs_eval {
          let subs = self.substitute(&body, param, &rhs_eval);
          self.evaluate_term(&subs)
        } else {
          // Cannot apply, construct the application with evaluated parts
          Term::Application {
            lhs: Box::new(lhs_eval),
            rhs: Box::new(rhs_eval),
          }
        }
      }
      // Otherwise, return the term as is
      _ => term.clone(),
    }
  }

  /// Substitute occurrences of a variable with a given term
  fn substitute(&mut self, term: &Term<'src>, var: &'src str, value: &Term<'src>) -> Term<'src> {
    match term {
      Term::Variable(name) if *name == var => value.clone(),
      Term::Variable(_) => term.clone(),
      Term::Abstraction { param, body } if *param != var => Term::Abstraction {
        param,
        body: Box::new(self.substitute(body, var, value)),
      },
      Term::Application { lhs, rhs } => Term::Application {
        lhs: Box::new(self.substitute(lhs, var, value)),
        rhs: Box::new(self.substitute(rhs, var, value)),
      },
      // nothing to substitute
      _ => term.clone(),
    }
  }
}

impl<'src> Evaluate<'src> for Interpreter {
  /// Simplify some term using α-conversion, β-reduction, and η-reduction
  fn evaluate(&mut self, term: &Term<'src>) -> Term<'src> {
    self.evaluate_term(term)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod interpreter {
    use super::*;

    #[test]
    fn evaluate_abstraction_identity() {
      let term = Term::Abstraction {
        param: "x",
        body: Box::new(Term::Variable("x")),
      };
      let mut interpreter = Interpreter;
      let result = interpreter.evaluate(&term);
      // Abstractions evaluate to themselves
      assert_eq!(result, term);
    }

    #[test]
    fn evaluate_simple_application() {
      let term = Term::Application {
        lhs: Box::new(Term::Abstraction {
          param: "x",
          body: Box::new(Term::Variable("x")),
        }),
        rhs: Box::new(Term::Variable("y")),
      };
      let mut interpreter = Interpreter;
      let result = interpreter.evaluate(&term);
      // (λx. x) y produces y
      assert_eq!(result, Term::Variable("y"));
    }

    #[test]
    fn evaluate_nested_application() {
      let term = Term::Application {
        lhs: Box::new(Term::Application {
          lhs: Box::new(Term::Abstraction {
            param: "x",
            body: Box::new(Term::Abstraction {
              param: "y",
              body: Box::new(Term::Variable("x")),
            }),
          }),
          rhs: Box::new(Term::Variable("a")),
        }),
        rhs: Box::new(Term::Variable("b")),
      };
      let mut interpreter = Interpreter;
      let result = interpreter.evaluate(&term);
      // ((λx. λy. x) a) b produces a
      assert_eq!(result, Term::Variable("a"));
    }

    #[test]
    fn evaluate_and() {
      // λp. λq. p q p
      let and_term = Term::Abstraction {
        param: "p",
        body: Box::new(Term::Abstraction {
          param: "q",
          body: Box::new(Term::Application {
            lhs: Box::new(Term::Application {
              lhs: Box::new(Term::Variable("p")),
              rhs: Box::new(Term::Variable("q")),
            }),
            rhs: Box::new(Term::Variable("p")),
          }),
        }),
      };
      // λx. λy. x
      let true_term = Term::Abstraction {
        param: "x",
        body: Box::new(Term::Abstraction {
          param: "y",
          body: Box::new(Term::Variable("x")),
        }),
      };
      // λx. λy. y
      let false_term = Term::Abstraction {
        param: "x",
        body: Box::new(Term::Abstraction {
          param: "y",
          body: Box::new(Term::Variable("y")),
        }),
      };

      // (λp. λq. p q p) (λx. λy. x) (λx. λy. y)
      let term = Term::Application {
        lhs: Box::new(Term::Application {
          lhs: Box::new(and_term),
          rhs: Box::new(true_term.clone()),
        }),
        rhs: Box::new(false_term.clone()),
      };

      let mut interpreter = Interpreter;
      let result = interpreter.evaluate(&term);
      // true AND false produces false
      assert_eq!(result, false_term);
    }

    #[test]
    fn evaluate_or() {
      // λp. λq. p p q
      let or_term = Term::Abstraction {
        param: "p",
        body: Box::new(Term::Abstraction {
          param: "q",
          body: Box::new(Term::Application {
            lhs: Box::new(Term::Application {
              lhs: Box::new(Term::Variable("p")),
              rhs: Box::new(Term::Variable("p")),
            }),
            rhs: Box::new(Term::Variable("q")),
          }),
        }),
      };
      // λx. λy. x
      let true_term = Term::Abstraction {
        param: "x",
        body: Box::new(Term::Abstraction {
          param: "y",
          body: Box::new(Term::Variable("x")),
        }),
      };
      // λx. λy. y
      let false_term = Term::Abstraction {
        param: "x",
        body: Box::new(Term::Abstraction {
          param: "y",
          body: Box::new(Term::Variable("y")),
        }),
      };

      // (λp. λq. p p q) (λx. λy. y) (λx. λy. x)
      let term = Term::Application {
        lhs: Box::new(Term::Application {
          lhs: Box::new(or_term),
          rhs: Box::new(false_term.clone()),
        }),
        rhs: Box::new(true_term.clone()),
      };

      let mut interpreter = Interpreter;
      let result = interpreter.evaluate(&term);
      // false or true produces true
      assert_eq!(result, true_term);
    }

    #[test]
    fn evaluate_not() {
      // λp. p (λx. λy. y) (λx. λy. x)
      let not_term = Term::Abstraction {
        param: "p",
        body: Box::new(Term::Application {
          lhs: Box::new(Term::Application {
            lhs: Box::new(Term::Variable("p")),
            rhs: Box::new(Term::Abstraction {
              param: "x",
              body: Box::new(Term::Abstraction {
                param: "y",
                body: Box::new(Term::Variable("y")),
              }),
            }),
          }),
          rhs: Box::new(Term::Abstraction {
            param: "x",
            body: Box::new(Term::Abstraction {
              param: "y",
              body: Box::new(Term::Variable("x")),
            }),
          }),
        }),
      };
      // λx. λy. x
      let true_term = Term::Abstraction {
        param: "x",
        body: Box::new(Term::Abstraction {
          param: "y",
          body: Box::new(Term::Variable("x")),
        }),
      };
      // λx. λy. y
      let false_term = Term::Abstraction {
        param: "x",
        body: Box::new(Term::Abstraction {
          param: "y",
          body: Box::new(Term::Variable("y")),
        }),
      };

      // (λp. p (λx. λy. y) (λx. λy. x)) (λx. λy. x)
      let term = Term::Application {
        lhs: Box::new(not_term),
        rhs: Box::new(true_term.clone()),
      };

      let mut interpreter = Interpreter;
      let result = interpreter.evaluate(&term);
      // NOT true produces false
      assert_eq!(result, false_term);
    }

    #[test]
    fn substitute_variable() {
      let term = Term::Variable("x");
      let mut interpreter = Interpreter;
      let substituted = interpreter.substitute(&term, "x", &Term::Variable("y"));
      // x[x := y] assigns to y
      assert_eq!(substituted, Term::Variable("y"));
    }

    #[test]
    fn substitute_abstraction_no_capture() {
      let term = Term::Abstraction {
        param: "x",
        body: Box::new(Term::Variable("x")),
      };
      let mut interpreter = Interpreter;
      let substituted = interpreter.substitute(&term, "y", &Term::Variable("z"));
      // λx. x[y := z] assigns to λx. x
      assert_eq!(substituted, term);
    }

    #[test]
    fn substitute_abstraction_capture_internal() {
      let term = Term::Abstraction {
        param: "x",
        body: Box::new(Term::Variable("y")),
      };
      let mut interpreter = Interpreter;
      let substituted = interpreter.substitute(&term, "y", &Term::Variable("z"));
      // λx. y[y := z] assigns to λx. z
      assert_eq!(
        substituted,
        Term::Abstraction {
          param: "x",
          body: Box::new(Term::Variable("z")),
        }
      );
    }

    #[test]
    fn substitute_application() {
      let term = Term::Application {
        lhs: Box::new(Term::Variable("x")),
        rhs: Box::new(Term::Variable("y")),
      };
      let mut interpreter = Interpreter;
      let substituted = interpreter.substitute(&term, "x", &Term::Variable("z"));
      // (x y)[x := z] assigns to z y
      assert_eq!(
        substituted,
        Term::Application {
          lhs: Box::new(Term::Variable("z")),
          rhs: Box::new(Term::Variable("y")),
        }
      );
    }
  }
}
