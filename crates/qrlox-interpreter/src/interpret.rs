use std::collections::HashMap;

use qrlox_syntax::{ast::Span, Binop, Expr, Spanned, Stmt};

use crate::{value::Value, Error};

macro_rules! bail {
    ($span:expr,  $($fmt:tt)*) => {
        return Err(ControlFlow::Error(err!($span, $($fmt)*)))
    };
}

macro_rules! err {
    ($span:expr,  $($fmt:tt)*) => {
        Error { message: format!($($fmt)*), span: $span.clone() }
    };
}

pub(crate) use {bail, err};

pub(crate) struct InterpreterCtx {
    globals: HashMap<String, Value>,
    scopes: Vec<HashMap<String, Value>>,
}

pub enum ControlFlow {
    Break,
    Continue,
    Error(Error),
}

impl InterpreterCtx {
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

    fn assign(&mut self, var: &str, value: Value) -> bool {
        let Self { globals, scopes } = self;

        for scope in std::iter::once(globals).chain(scopes).rev() {
            if let Some(ref_) = scope.get_mut(var) {
                *ref_ = value;
                return true;
            }
        }

        false
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

    pub fn eval_expr(&mut self, expr: &Spanned<Expr<'_>>) -> Result<Value, ControlFlow> {
        let s = expr.s.clone();

        Ok(match &expr.v {
            Expr::Number(n) => Value::Number(*n),
            Expr::String(s) => Value::String(s.to_string()),
            Expr::Boolean(b) => Value::Boolean(*b),
            Expr::Nil => Value::Nil,
            Expr::Not(e) => {
                let res = self.eval_expr(e)?;
                Value::Boolean(!res.is_truthy())
            }
            Expr::Neg(e) => {
                let res = self.eval_expr(e)?;
                Value::Number(f64::try_from(&res).or_else(|_| {
                    bail!(
                        e.s,
                        "Could not cast value of type {} to number",
                        res.type_().name()
                    )
                })?)
            }
            Expr::Binary(lhs, op, rhs) => self.eval_binop(expr.s.clone(), lhs, op, rhs)?,
            Expr::Var(var) => self
                .lookup(var)
                .ok_or(())
                .or_else(|_| bail!(s, "No variable '{}' has been declared", var))?,
            Expr::Assign(var, expr) => {
                let res = self.eval_expr(expr)?;
                let assigned = self.assign(var, res.clone());
                if !assigned {
                    bail!(
                        s,
                        "Could not assigned to '{}', it has not been declared.",
                        var
                    );
                }
                res
            }
            Expr::Call(callable, args) => {
                let callable = self.eval_expr(callable)?;
                let mut arg_values = Vec::with_capacity(args.len());
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }
                match callable {
                    Value::Native(native) => {
                        (native.f)(arg_values, s.clone()).map_err(ControlFlow::Error)?
                    }
                    _ => bail!(
                        s,
                        "Values of type {} cannot be called",
                        callable.type_().name()
                    ),
                }
            }
        })
    }

    fn eval_binop<'a>(
        &mut self,
        span: Span,
        lhs: &Spanned<Expr<'a>>,
        op: &Binop,
        rhs: &Spanned<Expr<'a>>,
    ) -> Result<Value, ControlFlow> {
        let lhs = self.eval_expr(lhs)?;

        match op {
            Binop::And => {
                if lhs.is_truthy() {
                    return self.eval_expr(rhs);
                }
            }
            Binop::Or => {
                if lhs.is_truthy() {
                    return Ok(lhs);
                } else {
                    return self.eval_expr(rhs);
                }
            }
            _ => {}
        }

        let rhs = self.eval_expr(rhs)?;

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
            Binop::And | Binop::Or => unreachable!("'and' and 'or' should be handled already"),
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

    pub fn exec_stmt(&mut self, stmt: &Spanned<Stmt<'_>>) -> Result<(), ControlFlow> {
        match &stmt.v {
            Stmt::Expr(expr) => {
                let _ = self.eval_expr(expr)?;
            }
            Stmt::Print(expr) => {
                let v = self.eval_expr(expr)?;
                println!("{}", v.repr());
            }
            Stmt::VarDecl(var, expr) => {
                let v = self.eval_expr(expr)?;
                self.declare(var.to_string(), v);
            }
            Stmt::Block(stmts) => {
                self.scoped(|ctx| -> Result<_, ControlFlow> {
                    for stmt in stmts {
                        ctx.exec_stmt(stmt)?;
                    }

                    Ok(())
                })?;
            }
            Stmt::If { cond, then, else_ } => {
                let cond = self.eval_expr(cond)?;

                if cond.is_truthy() {
                    self.scoped(|ctx| -> Result<_, ControlFlow> { ctx.exec_stmt(then) })?;
                } else if let Some(else_) = else_ {
                    self.scoped(|ctx| -> Result<_, ControlFlow> { ctx.exec_stmt(else_) })?;
                }
            }
            Stmt::While { cond, body } => {
                while self.eval_expr(cond)?.is_truthy() {
                    match self.exec_stmt(body) {
                        Ok(()) => {}
                        // Continuing is a no-op
                        Err(ControlFlow::Continue) => {}
                        Err(ControlFlow::Break) => break,
                        e @ Err(ControlFlow::Error(_)) => return e,
                    }
                }
            }
            Stmt::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                if let Some(initializer) = initializer {
                    self.exec_stmt(initializer)?;
                }

                loop {
                    let run = match condition {
                        Some(condition) => self.eval_expr(condition)?.is_truthy(),
                        None => true,
                    };

                    if !run {
                        break;
                    }

                    let body_res = self.exec_stmt(body);
                    match body_res {
                        Ok(()) => {}
                        Err(ControlFlow::Break) => break,
                        // Continuing is a no-op
                        Err(ControlFlow::Continue) => {}
                        e @ Err(ControlFlow::Error(_)) => return e,
                    }

                    if let Some(increment) = increment {
                        self.eval_expr(increment)?;
                    }
                }
            }
            Stmt::Break => return Err(ControlFlow::Break),
            Stmt::Continue => return Err(ControlFlow::Continue),
        };
        Ok(())
    }
}
