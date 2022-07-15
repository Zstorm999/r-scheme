use std::fmt;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Integer(i64),
    Symbol(String),
    LParen,
    RParen,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Integer(n) => write!(f, "{}", n),
            Token::Symbol(s) => write!(f, "{}", s),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
        }
    }
}

pub struct TokenIterator<'a> {
    chars: std::iter::Peekable<Chars<'a>>,
}

pub fn tokenize<'a>(s: &'a str) -> TokenIterator<'a> {
    TokenIterator {
        chars: s.chars().peekable(),
    }
}

impl Iterator for TokenIterator<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        println!("Calling next !");

        let mut current = self.chars.peek();

        println!("Peeked");
        println!("Current is none: {}", current.is_none());

        // consume every possible whitespace char, advancing to the next valid char
        while let Some(c) = current {
            if !c.is_whitespace() {
                break;
            }

            // current is whitespace, we consume it and peek to the next value
            self.chars.next();
            current = self.chars.peek();
        }

        let mut token = String::with_capacity(30);

        // read token
        while let Some(&c) = current {
            if c.is_whitespace() {
                break;
            }

            if c == '(' || c == ')' {
                // alone paren is a token by itself
                // we consume it only if it’s alone
                if token.is_empty() {
                    token.push(c);
                    self.chars.next();
                }
                break;
            }

            token.push(c);
            self.chars.next();
            current = self.chars.peek();
        }

        //parse token
        if token.is_empty() {
            return None;
        }

        match token.as_str() {
            "(" => return Some(Token::LParen),
            ")" => return Some(Token::RParen),
            _ => {
                if let Ok(i) = token.parse::<i64>() {
                    return Some(Token::Integer(i));
                } else {
                    return Some(Token::Symbol(token));
                }
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn if_lparen_parses_correctly() {
        let tokens: Vec<Token> = tokenize("(").collect();

        assert_eq!(tokens, vec![Token::LParen]);
    }

    #[test]
    fn if_rparen_parses_correctly() {
        let tokens: Vec<Token> = tokenize(")").collect();

        assert_eq!(tokens, vec![Token::RParen]);
    }

    #[test]
    fn if_integer_parses_correctly() {
        let tokens: Vec<Token> = tokenize("42").collect();

        assert_eq!(tokens, vec![Token::Integer(42)]);
    }

    #[test]
    fn if_symbol_parses_correctly() {
        let tokens: Vec<Token> = tokenize("keyword").collect();

        assert_eq!(tokens, vec![Token::Symbol("keyword".to_string())]);
    }

    #[test]
    fn if_sequence_parses_correctly() {
        let tokens: Vec<Token> = tokenize("(add 1 0)").collect();

        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::Symbol("add".to_string()),
                Token::Integer(1),
                Token::Integer(0),
                Token::RParen,
            ]
        );
    }
}
