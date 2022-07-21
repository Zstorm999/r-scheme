use crate::object::Object;
use std::cell::RefCell;
use std::rc::Rc;

use crate::env::{Env, EnvRef};

use crate::parser::parse;

pub fn eval(program: &str, env: &mut EnvRef) -> Result<Object, String> {
    match parse(program) {
        Ok(content) => eval_obj(&content, env),
        Err(err) => Err(format!("{}", err)),
    }
}

fn eval_obj(obj: &Object, env: &mut EnvRef) -> Result<Object, String> {
    match obj {
        Object::Bool(b) => Ok(Object::Bool(*b)),
        Object::Float(x) => Ok(Object::Float(*x)),
        Object::Integer(n) => Ok(Object::Integer(*n)),
        Object::Lambda(_params, _body) => Err("Lambda not yet evaluable".to_string()),
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
    match eval_builtin(list, env) {
        Some(res) => res,
        None => match list.get(0) {
            Some(Object::Symbol(name)) => eval_function_call(name, list, env),
            Some(_) => match list.into_iter().map(|o| eval_obj(o, env)).collect() {
                Ok(l) => Ok(Object::List(l)),
                Err(e) => Err(e),
            },
            None => Err("Empty list".to_string()),
        },
    }
}

fn eval_builtin(list: &Vec<Object>, env: &mut EnvRef) -> Option<Result<Object, String>> {
    const BINOPS: [&str; 8] = ["+", "-", "*", "/", "<", ">", "=", "!="];

    match &list[..] {
        [Object::Symbol(kw_define), Object::Symbol(s), value] if kw_define == "define" => {
            Some(define(&s, &value, env))
        }
        [Object::Symbol(op), left, right] if BINOPS.iter().any(|s| s == op) => {
            Some(binop(op, left, right, env))
        }
        [Object::Symbol(kw_if), cond, if_clause] if kw_if == "if" => {
            Some(eval_if(cond, if_clause, env))
        }
        [Object::Symbol(kw_if), cond, if_clause, else_clause] if kw_if == "if" => {
            Some(eval_if_else(cond, if_clause, else_clause, env))
        }
        [Object::Symbol(kw_lambda), Object::List(params_list), Object::List(body)]
            if kw_lambda == "lambda" =>
        {
            let mut params = Vec::with_capacity(list.len());

            for param in params_list {
                match param {
                    Object::Symbol(s) => params.push(s.clone()),
                    _ => return Some(Err(format!("Invalid lambda parameter: {}", param))),
                }
            }

            Some(Ok(Object::Lambda(params, body.clone())))
        }
        _ => None,
    }
}

fn define(symbol: &str, value: &Object, env: &mut EnvRef) -> Result<Object, String> {
    let val = eval_obj(value, env)?;

    env.borrow_mut().set(&symbol, val);
    Ok(Object::Bool(true))
}

fn binop(
    operator: &str,
    left: &Object,
    right: &Object,
    env: &mut EnvRef,
) -> Result<Object, String> {
    let left = &eval_obj(left, env)?;
    let right = &eval_obj(right, env)?;

    match operator {
        "+" => match (left, right) {
            (Object::Integer(l), Object::Integer(r)) => Ok(Object::Integer(l + r)),
            (Object::Integer(l), Object::Float(r)) => Ok(Object::Float(*l as f64 + r)),
            (Object::Float(l), Object::Integer(r)) => Ok(Object::Float(l + *r as f64)),
            (Object::Float(l), Object::Float(r)) => Ok(Object::Float(l + r)),
            _ => Err(format!(
                "Unable to apply binary operation on non-numeric values: ({} {} {})",
                operator, left, right
            )),
        },
        "-" => match (left, right) {
            (Object::Integer(l), Object::Integer(r)) => Ok(Object::Integer(l - r)),
            (Object::Integer(l), Object::Float(r)) => Ok(Object::Float(*l as f64 - r)),
            (Object::Float(l), Object::Integer(r)) => Ok(Object::Float(l - *r as f64)),
            (Object::Float(l), Object::Float(r)) => Ok(Object::Float(l - r)),
            _ => Err(format!(
                "Unable to apply binary operation on non-numeric values: ({} {} {})",
                operator, left, right
            )),
        },
        "*" => match (left, right) {
            (Object::Integer(l), Object::Integer(r)) => match i64::checked_mul(*l, *r) {
                Some(value) => Ok(Object::Integer(value)),
                None => Err("Integer overflow".to_string()),
            },
            (Object::Integer(l), Object::Float(r)) => Ok(Object::Float(*l as f64 * r)),
            (Object::Float(l), Object::Integer(r)) => Ok(Object::Float(l * *r as f64)),
            (Object::Float(l), Object::Float(r)) => Ok(Object::Float(l * r)),
            _ => Err(format!(
                "Unable to apply binary operation on non-numeric values: ({} {} {})",
                operator, left, right
            )),
        },
        "/" => match (left, right) {
            (Object::Integer(l), Object::Integer(r)) => Ok(Object::Integer(l / r)),
            (Object::Integer(l), Object::Float(r)) => Ok(Object::Float(*l as f64 / r)),
            (Object::Float(l), Object::Integer(r)) => Ok(Object::Float(l / *r as f64)),
            (Object::Float(l), Object::Float(r)) => Ok(Object::Float(l / r)),
            _ => Err(format!(
                "Unable to apply binary operation on non-numeric values: ({} {} {})",
                operator, left, right
            )),
        },
        "<" => match (left, right) {
            (Object::Integer(l), Object::Integer(r)) => Ok(Object::Bool(l < r)),
            (Object::Integer(l), Object::Float(r)) => Ok(Object::Bool((*l as f64) < *r)),
            (Object::Float(l), Object::Integer(r)) => Ok(Object::Bool(*l < *r as f64)),
            (Object::Float(l), Object::Float(r)) => Ok(Object::Bool(l < r)),
            _ => Err(format!(
                "Unable to apply binary operation on non-numeric values: ({} {} {})",
                operator, left, right
            )),
        },
        ">" => match (left, right) {
            (Object::Integer(l), Object::Integer(r)) => Ok(Object::Bool(l > r)),
            (Object::Integer(l), Object::Float(r)) => Ok(Object::Bool((*l as f64) > *r)),
            (Object::Float(l), Object::Integer(r)) => Ok(Object::Bool(*l > *r as f64)),
            (Object::Float(l), Object::Float(r)) => Ok(Object::Bool(l > r)),
            _ => Err(format!(
                "Unable to apply binary operation on non-numeric values: ({} {} {})",
                operator, left, right
            )),
        },
        "=" => match (left, right) {
            (Object::Integer(l), Object::Integer(r)) => Ok(Object::Bool(l == r)),
            (Object::Integer(l), Object::Float(r)) => Ok(Object::Bool((*l as f64) == *r)),
            (Object::Float(l), Object::Integer(r)) => Ok(Object::Bool(*l == *r as f64)),
            (Object::Float(l), Object::Float(r)) => Ok(Object::Bool(l == r)),
            _ => Err(format!(
                "Unable to apply binary operation on non-numeric values: ({} {} {})",
                operator, left, right
            )),
        },
        "!=" => match (left, right) {
            (Object::Integer(l), Object::Integer(r)) => Ok(Object::Bool(l != r)),
            (Object::Integer(l), Object::Float(r)) => Ok(Object::Bool((*l as f64) != *r)),
            (Object::Float(l), Object::Integer(r)) => Ok(Object::Bool(*l != *r as f64)),
            (Object::Float(l), Object::Float(r)) => Ok(Object::Bool(l != r)),
            _ => Err(format!(
                "Unable to apply binary operation on non-numeric values: ({} {} {})",
                operator, left, right
            )),
        }, // non-standard
        _ => unreachable!("Unknown binary operator !"),
    }
}

fn eval_condition(cond: &Object, env: &mut EnvRef) -> Result<bool, String> {
    match eval_obj(cond, env)? {
        Object::Bool(false) => Ok(false),
        _ => Ok(true),
    }
}

fn eval_if(condition: &Object, if_clause: &Object, env: &mut EnvRef) -> Result<Object, String> {
    if eval_condition(condition, env)? == true {
        eval_obj(if_clause, env)
    } else {
        Ok(Object::Bool(false))
    }
}

fn eval_if_else(
    condition: &Object,
    if_clause: &Object,
    else_clause: &Object,
    env: &mut EnvRef,
) -> Result<Object, String> {
    if eval_condition(condition, env)? == true {
        eval_obj(if_clause, env)
    } else {
        eval_obj(else_clause, env)
    }
}

fn eval_function_call(name: &str, list: &Vec<Object>, env: &mut EnvRef) -> Result<Object, String> {
    if list.is_empty() {
        return Err("Empty list".to_string());
    }

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
