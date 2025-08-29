use crate::statement::Statement;

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self
    }

    pub fn interpret(&mut self, statements: Vec<Statement>) {
        statements.iter().for_each(|s| {
            let _ = s.eval();
        });
    }
}
