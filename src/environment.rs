use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    lox_error,
    lox_type::{LoxNativeFunction, LoxNumber, LoxType},
};

macro_rules! lox_native_fn {
    ($arity:expr, $func:expr) => {{
        use std::sync::Arc;

        LoxType::Function(Arc::new(LoxNativeFunction {
            arity: $arity,
            body: Arc::new($func),
        }))
    }};
}

#[macro_export]
macro_rules! env {
    () => {
        crate::environment::shared_env().lock().unwrap()
    };
}

#[derive(Clone)]
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

static SHARED_ENV: OnceLock<Mutex<Environment>> = OnceLock::new();

pub fn shared_env() -> &'static Mutex<Environment> {
    SHARED_ENV.get_or_init(|| {
        let mut values = HashMap::new();

        let clock_fn = |_| {
            let now = SystemTime::now();

            let duration_since_epoch = now
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards lol.");

            return LoxType::Number(duration_since_epoch.as_millis() as LoxNumber);
        };

        values.insert("clock".to_string(), lox_native_fn!(0, clock_fn));

        Mutex::new(Environment {
            values,
            enclosing: None,
        })
    })
}

pub fn with_env<R>(env: &mut Environment, f: impl FnOnce() -> R) -> R {
    let prev = std::mem::replace(env, Environment::new());
    let new_env = Environment {
        values: HashMap::new(),
        enclosing: Some(Box::new(prev)),
    };
    *env = new_env;
    let result = f();
    if let Some(enclosing_box) = env.enclosing.take() {
        *env = *enclosing_box;
    }
    result
}
