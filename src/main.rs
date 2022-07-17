mod lexer;
mod object;
mod parser;

use lexer::{tokenize, Token};

fn main() {
    let tokens: Vec<Token> = tokenize("(").collect();

    for t in tokens {
        println!("{}", t);
    }
}
