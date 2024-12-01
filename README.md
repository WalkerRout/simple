# simple
Basic lambda calculus interpreter, very loosely based on Types and Programming Languages' Simply Typed Lambda-Calculus

### Demonstration
> see src/bin/main.rs

`((λx. λy. x) (λy. y)) (λx. x)` => `λy. y`
