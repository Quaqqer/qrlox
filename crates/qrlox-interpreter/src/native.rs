use qrlox_macros::interpreter_native;
use qrlox_syntax::ast::Span;

use crate::{
    cast::Cast,
    interpret::err,
    value::{Native, Value},
    Error,
};

struct InterpreterResult(Result<Value, Error>);

impl<T: Into<Value>> From<T> for InterpreterResult {
    fn from(value: T) -> Self {
        InterpreterResult(Ok(value.into()))
    }
}

impl<T: Into<Value>> From<Result<T, Error>> for InterpreterResult {
    fn from(value: Result<T, Error>) -> Self {
        InterpreterResult(value.map(Into::into))
    }
}

fn native_cast<T>(v: Value, arg_i: usize, span: &Span) -> Result<T, Error>
where
    Value: Cast<T>,
{
    let ty_name = v.type_().name();

    Cast::<T>::cast(&v).ok_or_else(|| {
        err!(
            span,
            "Could not cast argument {} of type {} to {}",
            arg_i,
            ty_name,
            <Value as Cast::<T>>::NAME
        )
    })
}

pub fn create_std() -> Vec<Native> {
    vec![
        clock(),
        sin(),
        cos(),
        tan(),
        asin(),
        acos(),
        atan(),
        atan2(),
    ]
}

#[interpreter_native]
fn clock() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time before unix epoch?!")
        .as_secs_f64()
}

#[interpreter_native]
fn sin(v: f64) -> f64 {
    v.sin()
}

#[interpreter_native]
fn cos(v: f64) -> f64 {
    v.cos()
}

#[interpreter_native]
fn tan(v: f64) -> f64 {
    v.tan()
}

#[interpreter_native]
fn asin(v: f64) -> f64 {
    v.asin()
}

#[interpreter_native]
fn acos(v: f64) -> f64 {
    v.acos()
}

#[interpreter_native]
fn atan(v: f64) -> f64 {
    v.atan()
}

#[interpreter_native]
fn atan2(y: f64, x: f64) -> f64 {
    f64::atan2(y, x)
}
