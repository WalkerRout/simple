use simple::interpreter::Interpreter;
use simple::lexer::Lexer;
use simple::parser::Parser;

fn main() {
  let input = r"(λx. (λy. (λz. x))) a b c";
  let lexer = Lexer::new(input);
  let mut parser = Parser::new(lexer);
  match parser.parse() {
    Ok(root) => {
      use simple::term::Evaluate;
      let mut interp = Interpreter;
      println!("original: {}", &root);
      println!("simplified: {}", interp.evaluate(&root));
    }
    Err(e) => println!("{e:?}"),
  }
}
