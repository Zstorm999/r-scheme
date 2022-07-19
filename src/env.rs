use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::object::Object;

pub type EnvRef = Rc<RefCell<Env>>;

#[derive(Default)]
pub struct Env {
    parent: Option<EnvRef>,
    vars: HashMap<String, Object>,
}

impl Env {
    pub fn new() -> Env {
        Default::default()
    }

    pub fn extend(parent: EnvRef) -> Env {
        Env {
            parent: Some(parent),
            vars: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        match self.vars.get(name) {
            Some(value) => Some(value.clone()),
            None => self
                .parent
                .as_ref()
                .and_then(|o| o.borrow().get(name).clone()),
        }
    }

    pub fn set(&mut self, name: &str, val: Object) {
        self.vars.insert(name.to_string(), val);
    }
}
