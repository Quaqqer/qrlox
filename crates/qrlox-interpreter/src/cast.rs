use crate::value::{Value, ValueType};

pub trait Cast<T> {
    const NAME: &'static str;

    fn cast(self) -> Option<T>;
}

impl Cast<bool> for Value {
    const NAME: &'static str = ValueType::Boolean.name();

    fn cast(self) -> Option<bool> {
        Some(self.is_truthy())
    }
}

impl Cast<f64> for Value {
    const NAME: &'static str = ValueType::Number.name();

    fn cast(self) -> Option<f64> {
        match self {
            Value::Number(v) => Some(v),
            Value::Boolean(b) => Some(if b { 1. } else { 0. }),
            _ => None,
        }
    }
}

impl Cast<String> for Value {
    const NAME: &'static str = ValueType::String.name();

    fn cast(self) -> Option<String> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }
}

impl Cast<()> for Value {
    const NAME: &'static str = ValueType::Nil.name();

    fn cast(self) -> Option<()> {
        match self {
            Value::Nil => Some(()),
            _ => None,
        }
    }
}
