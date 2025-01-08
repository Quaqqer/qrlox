use qrlox_macros::interpreter_native;
use qrlox_syntax::ast::Span;

use crate::{
    cast::Cast,
    interpret::{err, InterpreterCtx},
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

    Cast::<T>::cast(v).ok_or_else(|| {
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
    vec![clock()]
}

#[interpreter_native]
fn clock() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time before unix epoch?!")
        .as_secs_f64()
}
