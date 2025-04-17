use std::rc::Rc;

use qrlox_syntax::{ast::Span, Spanned, Stmt};

use crate::world::InterpreterWorld;

use super::Error;

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Native(Rc<Native>),
    Function(Rc<Function>),
}

pub struct Native {
    pub name: String,
    pub f: Box<dyn Fn(&mut dyn InterpreterWorld, &Span, Vec<Value>) -> Result<Value, Error>>,
}

#[derive(Debug)]
pub struct Function {
    pub params: Vec<String>,
    pub body: Vec<Spanned<Stmt>>,
}

impl std::fmt::Debug for Native {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}()", self.name)
    }
}

impl Value {
    pub fn type_(&self) -> ValueType {
        match self {
            Value::Nil => ValueType::Nil,
            Value::Boolean(_) => ValueType::Boolean,
            Value::Number(_) => ValueType::Number,
            Value::String(_) => ValueType::String,
            Value::Native(_) => ValueType::Function,
            Value::Function(_) => ValueType::Function,
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
            Value::Native(_) => "function".to_string(),
            Value::Function(_) => "function".to_string(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Boolean(false))
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Boolean(lhs), Value::Boolean(rhs)) => lhs == rhs,
            (Value::Number(lhs), Value::Number(rhs)) => lhs == rhs,
            (Value::String(lhs), Value::String(rhs)) => lhs == rhs,
            (Value::Native(lhs), Value::Native(rhs)) => Rc::ptr_eq(lhs, rhs),
            _ => false,
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
    Function,
}

impl ValueType {
    pub const fn name(&self) -> &'static str {
        match self {
            ValueType::Nil => "nil",
            ValueType::Boolean => "boolean",
            ValueType::Number => "number",
            ValueType::String => "string",
            ValueType::Function => "function",
        }
    }
}

impl From<()> for Value {
    fn from(_value: ()) -> Self {
        Value::Nil
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}
