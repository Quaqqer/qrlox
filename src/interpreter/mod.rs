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
    scopes: Vec<HashMap<String, Value>>,
}

impl Ctx {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            scopes: Vec::new(),
        }
    }

    fn declare(&mut self, var: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(var, value);
        } else {
            self.globals.insert(var, value);
        }
    }

    fn lookup(&mut self, var: &str) -> Option<Value> {
        let Self { globals, scopes } = self;

        for scope in std::iter::once(globals).chain(scopes).rev() {
            if let Some(value) = scope.get(var) {
                return Some(value.clone());
            }
        }

        None
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop().expect("Popped too many scopes");
    }

    fn scoped<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.enter_scope();
        let res = f(self);
        self.exit_scope();
        res
    }
}

pub fn eval_expr(ctx: &mut Ctx, expr: &Spanned<Expr<'_>>) -> Result<Value, Error> {
    let s = expr.s.clone();

    Ok(match &expr.v {
        Expr::Number(n) => Value::Number(*n),
        Expr::String(s) => Value::String(s.to_string()),
        Expr::Boolean(b) => Value::Boolean(*b),
        Expr::Nil => Value::Nil,
        Expr::Not(e) => {
            let res = eval_expr(ctx, e)?;
            Value::Boolean(!res.is_truthy())
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
        Stmt::Block(stmts) => {
            ctx.scoped(|ctx| -> Result<_, Error> {
                for stmt in stmts {
                    exec_stmt(ctx, stmt)?;
                }

                Ok(())
            })?;
        }
        Stmt::If { cond, then, else_ } => {
            let cond = eval_expr(ctx, cond)?;

            if cond.is_truthy() {
                println!("truthy");
                ctx.scoped(|ctx| -> Result<_, Error> {
                    for stmt in then.iter() {
                        exec_stmt(ctx, stmt)?;
                    }
                    Ok(())
                })?;
            } else if let Some(else_) = else_ {
                ctx.scoped(|ctx| -> Result<_, Error> {
                    for stmt in else_.iter() {
                        exec_stmt(ctx, stmt)?;
                    }
                    Ok(())
                })?;
            }
        }
    };
    Ok(())
}
