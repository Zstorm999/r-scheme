mod env;
mod eval;
mod lexer;
mod object;
mod parser;

use linefeed::{Interface, ReadResult};

use std::cell::RefCell;
use std::rc::Rc;

use env::Env;
use eval::eval;

const PROMPT: &str = "r-scheme> ";

fn main() -> std::io::Result<()> {
    let reader = Interface::new("r-scheme")?;

    reader.set_prompt(format!("{}", PROMPT).as_ref())?;

    let mut env = Rc::new(RefCell::new(Env::new()));

    while let ReadResult::Input(input) = reader.read_line()? {
        if input.eq("exit") {
            break;
        }

        let result = eval(input.as_ref(), &mut env);
        match result {
            Ok(v) => println!("{}", v),
            Err(err) => {
                if !err.is_empty() {
                    // empty error is just no input
                    println!("{}", err)
                }
            }
        }
    }

    println!("Goodbye");
    Ok(())
}
