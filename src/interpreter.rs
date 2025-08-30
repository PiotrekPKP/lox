use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    environment::Environment,
    lox_type::{LoxNumber, LoxType},
    statement::Statement,
};

macro_rules! lox_native_fn {
    ($arity:expr, $func:expr) => {{
        use crate::lox_type::LoxNativeFunction;
        use crate::lox_type::LoxType;
        use std::sync::Arc;

        LoxType::Function(Arc::new(LoxNativeFunction {
            arity: $arity,
            body: Arc::new($func),
        }))
    }};
}

pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut values = HashMap::new();

        let clock_fn = |_| {
            let now = SystemTime::now();

            let duration_since_epoch = now
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards lol.");

            return LoxType::Number(duration_since_epoch.as_millis() as LoxNumber);
        };

        values.insert("clock".to_string(), lox_native_fn!(0, clock_fn));

        let env = Environment::new(None, values);

        Self { env }
    }

    pub fn interpret(&mut self, statements: Vec<Statement>) {
        statements.iter().for_each(|s| {
            let _ = s.eval(&mut self.env);
        });
    }
}
