mod value;

use std::collections::HashMap;

use value::Value;

use crate::ast::{Binop, Expr, Span, Spanned, Stmt};

macro_rules! bail {
    ($span:expr,  $($args:tt)*) => {
        return Err(Error { message: format!($($args)*), span: $span.clone() })
    };
}

pub(crate) use bail;

pub struct Error {
    pub message: String,
    pub span: Span,
}

pub struct Ctx {
    globals: HashMap<String, Value>,
}

impl Ctx {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
        }
    }

    fn declare(&mut self, var: String, value: Value) {
        self.globals.insert(var, value);
    }

    fn lookup(&mut self, var: &str) -> Option<Value> {
        self.globals.get(var).cloned()
    }
}

pub fn eval_expr<'a>(ctx: &mut Ctx, expr: &Spanned<Expr<'a>>) -> Result<Value, Error> {
    let s = expr.s.clone();

    Ok(match &expr.v {
        Expr::Number(n) => Value::Number(*n),
        Expr::String(s) => Value::String(s.to_string()),
        Expr::Boolean(b) => Value::Boolean(*b),
        Expr::Nil => Value::Nil,
        Expr::Not(e) => {
            let res = eval_expr(ctx, e)?;
            Value::Boolean(bool::try_from(&res).or_else(|_| {
                bail!(
                    e.s,
                    "Could not cast value of type {} to boolean",
                    res.type_().name()
                )
            })?)
        }
        Expr::Neg(e) => {
            let res = eval_expr(ctx, e)?;
            Value::Number(f64::try_from(&res).or_else(|_| {
                bail!(
                    e.s,
                    "Could not cast value of type {} to number",
                    res.type_().name()
                )
            })?)
        }
        Expr::Binary(lhs, op, rhs) => eval_binop(ctx, expr.s.clone(), lhs, op, rhs)?,
        Expr::Var(var) => ctx
            .lookup(var)
            .ok_or(())
            .or_else(|_| bail!(s, "No variable '{}' has been declared", var))?,
    })
}

fn eval_binop<'a>(
    ctx: &mut Ctx,
    span: Span,
    lhs: &Spanned<Expr<'a>>,
    op: &Binop,
    rhs: &Spanned<Expr<'a>>,
) -> Result<Value, Error> {
    let lhs = eval_expr(ctx, lhs)?;
    let rhs = eval_expr(ctx, rhs)?;

    let res = match op {
        Binop::Eq => Some(Value::Boolean(lhs == rhs)),
        Binop::Ne => Some(Value::Boolean(lhs != rhs)),
        Binop::Lt => match (&lhs, &rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Some(Value::Boolean(lhs < rhs)),
            _ => None,
        },
        Binop::Le => match (&lhs, &rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Some(Value::Boolean(lhs <= rhs)),
            _ => None,
        },
        Binop::Gt => match (&lhs, &rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Some(Value::Boolean(lhs > rhs)),
            _ => None,
        },
        Binop::Ge => match (&lhs, &rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Some(Value::Boolean(lhs >= rhs)),
            _ => None,
        },
        Binop::Add => match (&lhs, &rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Some(Value::Number(lhs + rhs)),
            _ => None,
        },
        Binop::Sub => match (&lhs, &rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Some(Value::Number(lhs - rhs)),
            _ => None,
        },
        Binop::Mul => match (&lhs, &rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Some(Value::Number(lhs * rhs)),
            _ => None,
        },
        Binop::Div => match (&lhs, &rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Some(Value::Number(lhs / rhs)),
            _ => None,
        },
    };

    match res {
        Some(v) => Ok(v),
        None => bail!(
            span,
            "Operator {} is not supported for types {} and {}",
            op.name(),
            lhs.type_().name(),
            rhs.type_().name()
        ),
    }
}

pub fn exec_stmt(ctx: &mut Ctx, stmt: &Spanned<Stmt<'_>>) -> Result<(), Error> {
    match &stmt.v {
        Stmt::Expr(expr) => {
            let _ = eval_expr(ctx, expr)?;
        }
        Stmt::Print(expr) => {
            let v = eval_expr(ctx, expr)?;
            println!("{}", v.repr());
        }
        Stmt::VarDecl(var, expr) => {
            let v = eval_expr(ctx, expr)?;
            ctx.declare(var.to_string(), v);
        }
    };
    Ok(())
}
