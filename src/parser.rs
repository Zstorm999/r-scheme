use crate::lexer::{tokenize, Token, TokenIterator};
use crate::object::Object;

use std::fmt;

#[derive(Debug, Clone)]
pub struct ParseError {
    err: String,
}

impl ParseError {
    fn new(err: &str) -> ParseError {
        ParseError {
            err: err.to_string(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parse error: {}", self.err)
    }
}

pub fn parse(s: &str) -> Result<Object, ParseError> {
    let mut tokens = tokenize(s);

    parse_list(&mut tokens)
}

fn parse_list(tokens: &mut TokenIterator) -> Result<Object, ParseError> {
    fn parse_inner_list(tokens: &mut TokenIterator) -> Result<Object, ParseError> {
        let mut list: Vec<Object> = Vec::new();

        while let Some(t) = tokens.next() {
            match t {
                Token::Integer(n) => list.push(Object::Integer(n)),
                Token::Float(f) => list.push(Object::Float(f)),
                Token::String(s) => list.push(Object::String(s)),
                Token::Symbol(s) => list.push(Object::Symbol(s)),
                Token::LParen => {
                    let sub_list = parse_inner_list(tokens)?;
                    list.push(sub_list);
                }
                Token::RParen => return Ok(Object::List(list)),
                Token::LexerError(e) => return Err(ParseError::new(&e)),
            }
        }

        //no rparen has been encountered !
        Err(ParseError::new("Encountered an unexpected EOF !"))
    }

    let token = tokens.next();

    if token != Some(Token::LParen) {
        return Err(ParseError::new(&format!(
            "Excepted Left Paren, found {:?}",
            token
        )));
    }

    parse_inner_list(tokens)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn parse_integer() {
        let list = parse("(1)").unwrap();

        assert_eq!(list, Object::List(vec![Object::Integer(1)]))
    }

    #[test]
    fn parse_float() {
        let list = parse("(1.512)").unwrap();

        assert_eq!(list, Object::List(vec![Object::Float(1.512)]))
    }

    #[test]
    fn parse_string() {
        let list = parse("(\"Hello\")").unwrap();

        assert_eq!(
            list,
            Object::List(vec![Object::String("Hello".to_string())])
        )
    }

    #[test]
    fn parse_add() {
        let list = parse("(+ 1 2)").unwrap();

        assert_eq!(
            list,
            Object::List(vec![
                Object::Symbol("+".to_string()),
                Object::Integer(1),
                Object::Integer(2)
            ])
        )
    }

    #[test]
    fn parse_area_of_a_circle() {
        let program = "(
                         (define r 10)
                         (define pi 314)
                         (* pi (* r r))
                       )";
        let list = parse(program).unwrap();
        assert_eq!(
            list,
            Object::List(vec![
                Object::List(vec![
                    Object::Symbol("define".to_string()),
                    Object::Symbol("r".to_string()),
                    Object::Integer(10),
                ]),
                Object::List(vec![
                    Object::Symbol("define".to_string()),
                    Object::Symbol("pi".to_string()),
                    Object::Integer(314),
                ]),
                Object::List(vec![
                    Object::Symbol("*".to_string()),
                    Object::Symbol("pi".to_string()),
                    Object::List(vec![
                        Object::Symbol("*".to_string()),
                        Object::Symbol("r".to_string()),
                        Object::Symbol("r".to_string()),
                    ]),
                ]),
            ])
        );
    }
}
