#[derive(Debug, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl Value {
    pub fn type_(&self) -> ValueType {
        match self {
            Value::Nil => ValueType::Nil,
            Value::Boolean(_) => ValueType::Boolean,
            Value::Number(_) => ValueType::Number,
            Value::String(_) => ValueType::String,
        }
    }

    pub fn repr(&self) -> String {
        match self {
            Value::Nil => "nil".to_string(),
            Value::Boolean(b) => match b {
                true => "true".to_string(),
                false => "false".to_string(),
            },
            Value::Number(n) => n.to_string(),
            Value::String(s) => "\"".to_string() + s + "\"",
        }
    }
}

impl TryFrom<&Value> for bool {
    type Error = ();

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Boolean(b) => Ok(*b),
            _ => Err(()),
        }
    }
}

impl TryFrom<&Value> for f64 {
    type Error = ();

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => Ok(*n),
            _ => Err(()),
        }
    }
}

pub enum ValueType {
    Nil,
    Boolean,
    Number,
    String,
}

impl ValueType {
    pub fn name(&self) -> &'static str {
        match self {
            ValueType::Nil => "nil",
            ValueType::Boolean => "boolean",
            ValueType::Number => "number",
            ValueType::String => "string",
        }
    }
}
