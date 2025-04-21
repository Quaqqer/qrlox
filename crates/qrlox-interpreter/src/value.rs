use std::collections::HashMap;

use gc::{Finalize, Gc, GcCell, Trace};
use qrlox_compiler::Stmt;
use qrlox_syntax::{ast::Span, Spanned};

use crate::world::InterpreterWorld;

use super::Error;

#[derive(Debug, Clone, Trace, Finalize)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Native(Gc<Native>),
    Function(Gc<Function>),
    Class(Gc<Class>),
    Instance(Gc<GcCell<Instance>>),
}

#[derive(Trace, Finalize)]
pub struct Native {
    pub name: String,
    #[unsafe_ignore_trace]
    #[allow(clippy::type_complexity)]
    pub f: Box<dyn Fn(&mut dyn InterpreterWorld, &Span, Vec<Value>) -> Result<Value, Error>>,
}

impl std::fmt::Debug for Native {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}()", self.name)
    }
}

#[derive(Debug, Trace, Finalize)]
pub struct Function {
    pub n_params: usize,
    #[unsafe_ignore_trace]
    pub body: Vec<Spanned<Stmt>>,
}

#[derive(Debug, Clone, Trace, Finalize)]
pub struct Class {
    pub class_name: Gc<String>,
}

#[derive(Debug, Clone, Trace, Finalize)]
pub struct Instance {
    pub class: Gc<Class>,
    pub fields: HashMap<String, Value>,
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
            Value::Class(_) => ValueType::Class,
            Value::Instance(_) => ValueType::Instance,
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
            Value::Class(object) => object.class_name.to_string(),
            Value::Instance(instance) => {
                format!("{} instance", instance.borrow().class.class_name,)
            }
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
            (Value::Native(lhs), Value::Native(rhs)) => Gc::ptr_eq(lhs, rhs),
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
    Class,
    Instance,
}

impl ValueType {
    pub const fn name(&self) -> &'static str {
        match self {
            ValueType::Nil => "nil",
            ValueType::Boolean => "boolean",
            ValueType::Number => "number",
            ValueType::String => "string",
            ValueType::Function => "function",
            ValueType::Class => "class",
            ValueType::Instance => "instance",
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
