use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    lox_error,
    lox_type::{LoxNativeFunction, LoxNumber, LoxType},
};

#[derive(Clone)]
pub struct Environment {
    pub enclosing: Option<Box<Environment>>,
    pub values: HashMap<String, LoxType>,
}

impl Environment {
    pub fn new(enclosing: Option<Environment>, values: HashMap<String, LoxType>) -> Self {
        Self {
            enclosing: enclosing.map(Box::new),
            values,
        }
    }

    pub fn define(&mut self, name: String, value: LoxType) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &String) -> &LoxType {
        if let Some(value) = self.values.get(name) {
            return value;
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.get(name);
        }

        lox_error!("Undefined variable '{}'.", name);
    }

    pub fn assign(&mut self, name: String, value: LoxType) {
        if let Some(_) = self.values.get(&name) {
            self.values.insert(name.clone(), value);
            return;
        }

        if let Some(ref mut enclosing) = self.enclosing {
            enclosing.assign(name.clone(), value);
            return;
        }

        lox_error!("Undefined variable '{}'", name);
    }
}
