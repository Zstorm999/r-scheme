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

    println!("Welcome to r-scheme — Scheme r7 (incomplete)");
    println!(
        "If you have any problem, please fill an issue at https://github.com/Zstorm999/r-scheme"
    );
    println!("Type \"exit\" to exit the interpreter");

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

        reader.add_history_unique(input);
    }

    println!("Goodbye");
    Ok(())
}
