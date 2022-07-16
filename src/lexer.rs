use std::fmt;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Integer(i64),
    Float(f64),
    String(String),
    Symbol(String),
    LParen,
    RParen,
    LexerError(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Integer(n) => write!(f, "{}", n),
            Token::Float(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Symbol(s) => write!(f, "{}", s),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LexerError(s) => write!(f, "{}", s),
        }
    }
}

pub struct TokenIterator<'a> {
    chars: std::iter::Peekable<Chars<'a>>,
    is_error: bool,
    c_count: usize,
    line_count: usize,
}

pub fn tokenize<'a>(s: &'a str) -> TokenIterator<'a> {
    TokenIterator {
        chars: s.chars().peekable(),
        is_error: false,
        c_count: 1,
        line_count: 1,
    }
}

impl Iterator for TokenIterator<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        if self.is_error {
            return None;
        }

        let mut current = self.chars.peek();

        // consume every possible whitespace char, advancing to the next valid char
        while let Some(&c) = current {
            if !c.is_whitespace() {
                if c == '\n' {
                    self.line_count += 1;
                    self.c_count = 0;
                }

                break;
            }

            // current is whitespace, we consume it and peek to the next value
            self.chars.next();
            self.c_count += 1;
            current = self.chars.peek();
        }

        let mut token = String::with_capacity(30);
        let mut in_string = false;

        // read token
        while let Some(&c) = current {
            if in_string {
                if c == '\n' {
                    self.line_count += 1;
                }

                if c == '"' {
                    // end of the string
                    self.chars.next();
                    self.c_count += 1;
                    current = self.chars.peek();

                    if let Some(&c) = current {
                        // we check that the string is not immediately followed by another token, which is an error
                        if !c.is_whitespace() && c != '(' && c != ')' {
                            self.is_error = true;
                            return Some(Token::LexerError(format!(
                                "Unexpected character {} (line {}, column {})",
                                c, self.line_count, self.c_count
                            )));
                        }
                    }

                    // at this point the string literal is valid
                    return Some(Token::String(token));
                }

                token.push(c);
                self.chars.next();
                self.c_count += 1;
                current = self.chars.peek();
            } else {
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

                if c == '"' {
                    // beginning of a string

                    if token.is_empty() {
                        in_string = true;
                        self.chars.next();
                        self.c_count += 1;
                        current = self.chars.peek();
                        continue;
                    } else {
                        self.is_error = true;

                        return Some(Token::LexerError(format!(
                            "Unexpected character \" (line {}, column {})",
                            self.line_count, self.c_count
                        )));
                    }
                }

                //part of a normal token
                token.push(c);
                self.chars.next();
                self.c_count += 1;
                current = self.chars.peek();
            }
        }

        // check that the string literal has been closed
        if in_string {
            return Some(Token::LexerError(format!(
                "Unexpected EOF (line {}, column {})",
                self.line_count, self.c_count
            )));
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
                } else if let Ok(f) = token.parse::<f64>() {
                    return Some(Token::Float(f));
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
    fn integer_parses_correctly() {
        let tokens: Vec<Token> = tokenize("42").collect();

        assert_eq!(tokens, vec![Token::Integer(42)]);
    }

    #[test]
    fn float_parses_correctly() {
        let tokens: Vec<Token> = tokenize("42.42").collect();

        assert_eq!(tokens, vec![Token::Float(42.42)]);
    }

    //todo: add tests for string
    #[test]
    fn empty_string_parses_correctly() {
        let tokens: Vec<Token> = tokenize("\"\"").collect();

        assert_eq!(tokens, vec![Token::String("".to_string())]);
    }

    #[test]
    fn string_parses_correctly() {
        let tokens: Vec<Token> = tokenize("\"Hello it this the Crusty Crab ?\"").collect();

        assert_eq!(
            tokens,
            vec![Token::String("Hello it this the Crusty Crab ?".to_string())]
        );
    }

    #[test]
    fn symbol_concat_string_parses_error() {
        let tokens: Vec<Token> = tokenize("format\"No this is Patrick\"").collect();

        assert_eq!(
            tokens,
            vec![Token::LexerError(
                "Unexpected character \" (line 1, column 7)".to_string()
            )]
        );
    }

    #[test]
    fn string_concat_symbol_parses_error() {
        let tokens: Vec<Token> = tokenize("\"Oh ok\"*Leaves_silently").collect();

        assert_eq!(
            tokens,
            vec![Token::LexerError(
                "Unexpected character * (line 1, column 8)".to_string()
            )]
        );
    }

    #[test]
    fn symbol_parses_correctly() {
        let tokens: Vec<Token> = tokenize("keyword").collect();

        assert_eq!(tokens, vec![Token::Symbol("keyword".to_string())]);
    }

    #[test]
    fn lparen_parses_correctly() {
        let tokens: Vec<Token> = tokenize("(").collect();

        assert_eq!(tokens, vec![Token::LParen]);
    }

    #[test]
    fn rparen_parses_correctly() {
        let tokens: Vec<Token> = tokenize(")").collect();

        assert_eq!(tokens, vec![Token::RParen]);
    }

    #[test]
    fn sequence_parses_correctly() {
        let tokens: Vec<Token> = tokenize("(add 1 (convert (\"125\")))").collect();

        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::Symbol("add".to_string()),
                Token::Integer(1),
                Token::LParen,
                Token::Symbol("convert".to_string()),
                Token::LParen,
                Token::String("125".to_string()),
                Token::RParen,
                Token::RParen,
                Token::RParen,
            ]
        );
    }
}
