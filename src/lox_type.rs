pub type LoxString = String;
pub type LoxNumber = f64;
pub type LoxBoolean = bool;

#[derive(Debug, Clone)]
pub enum LoxType {
    String(LoxString),
    Number(LoxNumber),
    Boolean(LoxBoolean),
    Nil,
    Unknown,
}

impl LoxType {
    pub fn is_truthy(&self) -> bool {
        match self {
            LoxType::Boolean(b) => *b,
            LoxType::Nil => false,
            LoxType::Number(n) => *n != 0.,
            _ => true,
        }
    }
}

impl std::fmt::Display for LoxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxType::Boolean(b) => write!(f, "{b}"),
            LoxType::Nil => write!(f, "nil"),
            LoxType::Number(n) => write!(f, "{n}"),
            LoxType::String(s) => write!(f, "{s}"),
            LoxType::Unknown => write!(f, "\0"),
        }
    }
}

impl PartialEq for LoxType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
