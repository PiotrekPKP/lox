use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    lox_error,
    lox_type::{LoxFunction, LoxNumber, LoxType},
    statement::Statement,
};

#[derive(Debug, Clone)]
pub struct Environment {
    pub enclosing: Option<Box<Environment>>,
    pub values: HashMap<String, LoxType>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
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

static GLOBAL_ENV: OnceLock<Mutex<Environment>> = OnceLock::new();

pub fn global_env() -> &'static Mutex<Environment> {
    GLOBAL_ENV.get_or_init(|| {
        let mut values = HashMap::new();

        values.insert(
            "clock".to_string(),
            LoxType::Function(LoxFunction {
                arity: 0,
                body: Statement::NativeFn(Arc::new(|| {
                    let now = SystemTime::now();

                    let duration_since_epoch = now
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards lol.");

                    return LoxType::Number(duration_since_epoch.as_millis() as LoxNumber);
                })),
            }),
        );

        Mutex::new(Environment {
            values,
            enclosing: None,
        })
    })
}
