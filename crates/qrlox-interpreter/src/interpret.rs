use std::{collections::HashMap, rc::Rc};

use qrlox_compiler::{Binop, ClassDecl, Expr, FunDecl, Ident, Stmt};
use qrlox_syntax::{ast::Span, Spanned};

use crate::{
    value::{Class, Function, Instance, Native, Value},
    world::InterpreterWorld,
    Error,
};

macro_rules! bail {
    ($span:expr,  $($fmt:tt)*) => {
        return Err($crate::interpret::ControlFlow::Error(err!($span, $($fmt)*)))
    };
}

macro_rules! err {
    ($span:expr,  $($fmt:tt)*) => {
        Error { message: format!($($fmt)*), span: $span.clone() }
    };
}

pub(crate) use {bail, err};

pub struct InterpreterCtx<World>
where
    World: InterpreterWorld,
{
    globals: HashMap<String, Value>,
    environments: Vec<Vec<Value>>,
    pub world: World,
}

pub enum ControlFlow {
    Break,
    Continue,
    Error(Error),
    Return(Value),
}

impl<World> InterpreterCtx<World>
where
    World: InterpreterWorld,
{
    #[allow(clippy::new_without_default)]
    pub fn new(world: World) -> Self {
        Self {
            globals: HashMap::new(),
            environments: vec![Vec::new()],
            world,
        }
    }

    pub fn add_native(&mut self, native: Native) {
        self.declare(
            &Ident::Global(Rc::new(native.name.to_string())),
            Value::Native(Rc::new(native)),
        );
    }

    fn declare(&mut self, ident: &Ident, value: Value) {
        match ident {
            Ident::Global(name) => {
                self.globals.insert(name.to_string(), value);
            }
            Ident::Local(d) => {
                debug_assert!(self.env().len() == *d);
                self.env_mut().push(value);
            }
        }
    }

    fn lookup(&self, ident: &Ident) -> Option<Value> {
        match ident {
            Ident::Global(name) => self.globals.get(name.as_str()).cloned(),
            Ident::Local(d) => self.env().get(*d).cloned(),
        }
    }

    fn assign(&mut self, ident: &Ident, value: Value) -> bool {
        let slot = match ident {
            Ident::Global(name) => self.globals.get_mut(name.as_str()),
            Ident::Local(d) => self.env_mut().get_mut(*d),
        };

        if let Some(slot) = slot {
            *slot = value;
            true
        } else {
            false
        }
    }

    fn enter_env(&mut self) {
        self.environments.push(Vec::new());
    }

    fn exit_env(&mut self) {
        self.environments
            .pop()
            .expect("Popped too many environments");
    }

    fn with_env<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.enter_env();
        let res = f(self);
        self.exit_env();
        res
    }

    #[allow(unused)]
    fn env(&self) -> &Vec<Value> {
        self.environments.last().unwrap()
    }

    fn env_mut(&mut self) -> &mut Vec<Value> {
        self.environments.last_mut().unwrap()
    }

    pub fn eval_expr(&mut self, expr: &Spanned<Expr>) -> Result<Value, ControlFlow> {
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
            Expr::Var(var) => self.lookup(var).ok_or(()).or_else(|_| {
                if let Ident::Global(name) = var {
                    bail!(s, "No global variable '{}' has been declared", name);
                } else {
                    bail!(s, "Variable has not been initialized yet.");
                }
            })?,
            Expr::Assign(var, expr) => {
                let res = self.eval_expr(expr)?;
                let assigned = self.assign(var, res.clone());
                if !assigned {
                    if let Ident::Global(name) = var {
                        bail!(
                            s,
                            "Could not assign to '{}', it has not been declared.",
                            name
                        );
                    } else {
                        unreachable!(
                            "A variable that hasn't been declared yet cannot be assigned to."
                        );
                    }
                }
                res
            }
            Expr::Call(callable, args) => {
                let callable_v = self.eval_expr(callable)?;
                let mut arg_values = Vec::with_capacity(args.len());
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }
                match callable_v {
                    Value::Native(native) => {
                        (native.f)(&mut self.world, &s, arg_values).map_err(ControlFlow::Error)?
                    }
                    Value::Function(fun) => self.with_env(|ctx| -> Result<Value, ControlFlow> {
                        if arg_values.len() != fun.n_params {
                            bail!(
                                s,
                                "Expected {} arguments, got {}",
                                fun.n_params,
                                arg_values.len()
                            );
                        }

                        for (i, val) in arg_values.into_iter().enumerate() {
                            ctx.declare(&Ident::Local(i), val);
                        }

                        for stmt in &fun.body {
                            let res = ctx.exec_stmt(stmt);
                            match res {
                                Ok(()) => {}
                                Err(ControlFlow::Break) => {
                                    bail!(stmt.s, "Cannot break out of a function")
                                }
                                Err(ControlFlow::Continue) => {
                                    bail!(stmt.s, "Cannot continue out of a function")
                                }
                                Err(ControlFlow::Return(v)) => return Ok(v),
                                Err(e @ ControlFlow::Error(_)) => return Err(e),
                            }
                        }
                        Ok(Value::Nil)
                    })?,
                    Value::Class(class) => {
                        let expected_arguments = 0;
                        if arg_values.len() != expected_arguments {
                            bail!(
                                callable.s,
                                "Constructor for class '{}' expected {} arguments",
                                class.class_name,
                                expected_arguments
                            );
                        }

                        Value::Instance(Instance {
                            class: class.clone(),
                        })
                    }
                    _ => bail!(
                        callable.s,
                        "Values of type {} cannot be called",
                        callable_v.type_().name()
                    ),
                }
            }
            Expr::Fun(params, body) => Value::Function(Rc::new(Function {
                n_params: params.len(),
                body: body.clone(),
            })),
        })
    }

    fn eval_binop(
        &mut self,
        span: Span,
        lhs: &Spanned<Expr>,
        op: &Binop,
        rhs: &Spanned<Expr>,
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
                (Value::String(lhs), Value::String(rhs)) => Some(Value::String(lhs.clone() + rhs)),
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

    pub fn exec_stmt(&mut self, stmt: &Spanned<Stmt>) -> Result<(), ControlFlow> {
        let s = &stmt.s;

        match &stmt.v {
            Stmt::Expr(expr) => {
                let _ = self.eval_expr(expr)?;
            }
            Stmt::Print(expr) => {
                let v = self.eval_expr(expr)?;

                if let Value::String(s) = v {
                    self.world.println(&s);
                } else {
                    self.world.println(&v.repr());
                }
            }
            Stmt::VarDecl(var, expr) => {
                let v = self.eval_expr(expr)?;
                self.declare(var, v);
            }
            Stmt::Block(stmts, remaining_variables) => {
                let mut res = Ok(());
                for stmt in stmts {
                    match self.exec_stmt(stmt) {
                        Ok(()) => {}
                        Err(e) => {
                            res = Err(e);
                            break;
                        }
                    }
                }

                for _ in *remaining_variables..self.env().len() {
                    self.env_mut().pop().unwrap();
                }

                res?;
            }
            Stmt::If { cond, then, else_ } => {
                let cond = self.eval_expr(cond)?;

                if cond.is_truthy() {
                    self.exec_stmt(then)?;
                } else if let Some(else_) = else_ {
                    self.exec_stmt(else_)?;
                }
            }
            Stmt::While { cond, body } => {
                while self.eval_expr(cond)?.is_truthy() {
                    match self.exec_stmt(body) {
                        Ok(()) => {}
                        // Continuing is a no-op
                        Err(ControlFlow::Continue) => {}
                        Err(ControlFlow::Break) => break,
                        e @ Err(ControlFlow::Error(_) | ControlFlow::Return(_)) => return e,
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
                        e @ Err(ControlFlow::Error(_) | ControlFlow::Return(_)) => return e,
                    }

                    if let Some(increment) = increment {
                        self.eval_expr(increment)?;
                    }
                }
            }
            Stmt::Break => return Err(ControlFlow::Break),
            Stmt::Continue => return Err(ControlFlow::Continue),
            Stmt::FunDecl(FunDecl {
                name,
                n_params,
                body,
            }) => {
                self.declare(
                    name,
                    Value::Function(Rc::new(Function {
                        n_params: *n_params,
                        body: body.clone(),
                    })),
                );
            }
            Stmt::ClassDecl(ClassDecl {
                class_name,
                ident,
                functions,
            }) => self.declare(
                ident,
                Value::Class(Rc::new(Class {
                    class_name: class_name.clone(),
                })),
            ),
            Stmt::Return(expr) => {
                if self.environments.len() == 1 {
                    bail!(s, "Tried to return outside of a function call.")
                }
                return Err(ControlFlow::Return(self.eval_expr(expr)?));
            }
        };
        Ok(())
    }
}
