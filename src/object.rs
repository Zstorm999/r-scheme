use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Symbol(String),
    Lambda(Vec<String>, Vec<Object>),
    List(Vec<Object>),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Integer(n) => write!(f, "{}", n),
            Object::Float(fl) => write!(f, "{}", fl),
            Object::Bool(b) => write!(f, "{}", b),
            Object::String(s) => write!(f, "\"{}\"", s),
            Object::Symbol(s) => write!(f, "{}", s),
            Object::Lambda(params, body) => {
                write!(f, "λ ")?;

                for p in params {
                    write!(f, "{} ", p)?;
                }
                write!(f, ". ")?;

                for e in body {
                    write!(f, "{} ", e)?;
                }

                Ok(())
            }
            Object::List(list) => {
                write!(f, "(")?;

                for (i, o) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", o)?;
                }

                Ok(())
            }
        }
    }
}
