use crate::object::Object;
use std::cell::RefCell;
use std::rc::Rc;

use crate::env::{Env, EnvRef};

use crate::parser::{parse, ParseError};

pub fn eval(program: &str, env: &mut EnvRef) -> Result<Object, String> {
    match parse(program) {
        Ok(content) => eval_obj(&content, env),
        Err(err) => Err(format!("{}", err)),
    }
}

fn eval_obj(obj: &Object, env: &mut EnvRef) -> Result<Object, String> {
    match obj {
        Object::Bool(_) => Ok(obj.clone()),
        Object::Float(x) => Ok(Object::Float(*x)),
        Object::Integer(n) => Ok(Object::Integer(*n)),
        Object::Lambda(_params, _body) => Err("Lamda not yet evaluable".to_string()),
        Object::List(list) => eval_list(list, env),
        Object::String(s) => Ok(Object::String(s.to_string())),
        Object::Symbol(s) => eval_symbol(s, env),
    }
}

fn eval_symbol(s: &str, env: &mut EnvRef) -> Result<Object, String> {
    let val = env.borrow_mut().get(s);

    if val.is_none() {
        return Err(format!("Unbound symbol {}", s));
    }

    Ok(val.unwrap().clone())
}

fn eval_list(list: &Vec<Object>, env: &mut EnvRef) -> Result<Object, String> {
    let head = match list.get(0) {
        Some(o) => o,
        None => return Err("Empty list".to_string()),
    };

    match head {
        Object::Symbol(s) => match s.as_str() {
            "define" => eval_define(&list, env),
            "+" | "-" | "*" | "/" | "<" | ">" | "=" | "!=" => {
                return eval_binop(&list, env);
            }
            "if" => eval_if(&list, env),
            "lambda" => eval_lambda(&list),
            _ => eval_function_call(&s, &list, env),
        },
        _ => match list.into_iter().map(|o| eval_obj(o, env)).collect() {
            Ok(l) => Ok(Object::List(l)),
            Err(e) => Err(e),
        },
    }
}

fn eval_define(list: &Vec<Object>, env: &mut EnvRef) -> Result<Object, String> {
    if list.len() != 3 {
        return Err(format!(
            "define expression requires 2 arguments, got {}",
            list.len() - 1
        ));
    }
    let symbol = match &list[1] {
        Object::Symbol(s) => s,
        _ => return Err("Invalid symbol in define expression".to_string()),
    };

    let val = eval_obj(&list[2], env)?;

    env.borrow_mut().set(&symbol, val);

    Ok(Object::Bool(true))
}

fn eval_binop(list: &Vec<Object>, env: &mut EnvRef) -> Result<Object, String> {
    if list.len() != 3 {
        return Err(format!(
            "Expected 2 arguments to binary operation {}",
            &list[0]
        ));
    }

    let operator = &list[0];
    let left = eval_obj(&list[1], env)?;
    let right = eval_obj(&list[2], env)?;

    let left = match left {
        Object::Integer(n) => n,
        _ => return Err(format!("Left operand is not an integer: {}", left)),
    };

    let right = match right {
        Object::Integer(n) => n,
        _ => return Err(format!("Right operand is not an integer: {}", right)),
    };

    if let Object::Symbol(s) = operator {
        match s.as_str() {
            "+" => Ok(Object::Integer(left + right)),
            "-" => Ok(Object::Integer(left - right)),
            "*" => Ok(Object::Integer(left * right)),
            "/" => Ok(Object::Integer(left / right)),
            "<" => Ok(Object::Bool(left < right)),
            ">" => Ok(Object::Bool(left > right)),
            "=" => Ok(Object::Bool(left == right)),
            "!=" => Ok(Object::Bool(left != right)), // non-standard
            _ => unreachable!("Unknown binary operator !"),
        }
    } else {
        Err("Operator must be a symbol".to_string())
    }
}

fn eval_if(list: &Vec<Object>, env: &mut EnvRef) -> Result<Object, String> {
    if list.len() != 3 && list.len() != 4 {
        return Err(format!(
            "if expression requires 2 or 3 arguments, got {}",
            list.len() - 1
        ));
    }

    let condition = eval_obj(&list[1], env)?;

    let condition = match condition {
        // #f is the only scheme value evaluating to #f
        Object::Bool(false) => false,
        _ => true,
    };

    if condition == true {
        eval_obj(&list[2], env)
    } else {
        if let Some(else_clause) = list.get(3) {
            eval_obj(else_clause, env)
        } else {
            // unspecified in the specs
            Ok(Object::Bool(false))
        }
    }
}

fn eval_lambda(list: &Vec<Object>) -> Result<Object, String> {
    if list.len() != 3 {
        return Err(format!(
            "lambda expression requires 2 arguments, got {}",
            list.len() - 1
        ));
    }

    let params = match &list[1] {
        Object::List(list) => {
            let mut params = Vec::with_capacity(list.len());

            for param in list {
                match param {
                    Object::Symbol(s) => params.push(s.clone()),
                    _ => return Err(format!("Invalid lambda parameter: {}", param)),
                }
            }
            params
        }
        _ => {
            return Err(format!(
                "First argument of lambda expression is not a list: {}",
                &list[1]
            ))
        }
    };

    let body = match &list[2] {
        Object::List(list) => list.clone(),
        _ => {
            return Err(format!(
                "Second argument of lambda expression is not a list: {}",
                &list[2]
            ))
        }
    };

    Ok(Object::Lambda(params, body))
}

fn eval_function_call(name: &str, list: &Vec<Object>, env: &mut EnvRef) -> Result<Object, String> {
    let function = env.borrow_mut().get(name);

    if function.is_none() {
        return Err(format!("Unbound symbol: {}", name));
    }

    match function {
        Some(Object::Lambda(params, body)) => {
            if params.len() != list.len() - 1 {
                return Err(format!(
                    "Lambda expects {} parameters, but was given {}",
                    params.len(),
                    list.len() - 1
                ));
            }

            let mut new_env = Rc::new(RefCell::new(Env::extend(env.clone())));

            // not lazy
            for (i, p) in params.iter().enumerate() {
                let val = eval_obj(&list[i + 1], env)?; // evaluate parameters in the parent scope
                new_env.borrow_mut().set(p, val);
            }
            eval_obj(&Object::List(body), &mut new_env)
        }
        _ => Err(format!(
            "Trying to evaluate non-function expression: {}",
            name
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_add() {
        let mut env = Rc::new(RefCell::new(Env::new()));
        let result = eval("(+ 1 2)", &mut env).unwrap();
        assert_eq!(result, Object::Integer(3));
    }

    #[test]
    fn test_area_of_a_circle() {
        let mut env = Rc::new(RefCell::new(Env::new()));
        let program = "(
                        (define r 10)
                        (define pi 314)
                        (* pi (* r r))
                      )";
        let result = eval(program, &mut env).unwrap();
        assert_eq!(
            result,
            Object::List(vec![
                Object::Bool(true),
                Object::Bool(true),
                Object::Integer((314 * 10 * 10) as i64)
            ])
        );
    }

    #[test]
    fn test_sqr_function() {
        let mut env = Rc::new(RefCell::new(Env::new()));
        let program = "(
                        (define sqr (lambda (r) (* r r))) 
                        (sqr 10)
                       )";
        let result = eval(program, &mut env).unwrap();
        assert_eq!(
            result,
            Object::List(vec![Object::Bool(true), Object::Integer((10 * 10) as i64)])
        );
    }

    #[test]
    fn test_fibonaci() {
        let mut env = Rc::new(RefCell::new(Env::new()));
        let program = "
            (
                (define fib (lambda (n) (if (< n 2) 1 (+ (fib (- n 1)) (fib (- n 2))))))
                (fib 10)
            )
        ";

        let result = eval(program, &mut env).unwrap();
        assert_eq!(
            result,
            Object::List(vec![Object::Bool(true), Object::Integer((89) as i64)])
        );
    }

    #[test]
    fn test_factorial() {
        let mut env = Rc::new(RefCell::new(Env::new()));
        let program = "
            (
                (define fact (lambda (n) (if (< n 1) 1 (* n (fact (- n 1))))))
                (fact 5)
            )
        ";

        let result = eval(program, &mut env).unwrap();
        assert_eq!(
            result,
            Object::List(vec![Object::Bool(true), Object::Integer((120) as i64)])
        );
    }

    #[test]
    fn test_circle_area_function() {
        let mut env = Rc::new(RefCell::new(Env::new()));
        let program = "
            (
                (define pi 314)
                (define r 10)
                (define sqr (lambda (r) (* r r)))
                (define area (lambda (r) (* pi (sqr r))))
                (area r)
            )
        ";

        let result = eval(program, &mut env).unwrap();
        assert_eq!(
            result,
            Object::List(vec![
                Object::Bool(true),
                Object::Bool(true),
                Object::Bool(true),
                Object::Bool(true),
                Object::Integer((314 * 10 * 10) as i64)
            ])
        );
    }
}
