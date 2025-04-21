use std::{collections::HashMap, rc::Rc};

use qrlox_syntax::{
    Spanned,
    ast::{self, Span},
};

#[derive(Debug)]
pub struct Error {
    message: String,
    span: Span,
}

struct Scope {
    variables: HashMap<String, usize>,
    depth: usize,
}

struct Environment {
    scopes: Vec<Scope>,
}

impl Environment {
    fn new_program() -> Self {
        Self { scopes: Vec::new() }
    }

    fn new_closure() -> Self {
        Self {
            scopes: vec![Scope {
                variables: HashMap::new(),
                depth: 0,
            }],
        }
    }
}

struct Resolver {
    environments: Vec<Environment>,
}

pub fn resolve_program(
    program: &Vec<Spanned<ast::Stmt>>,
    ariadne_config: &ariadne::Config,
) -> Result<Vec<Spanned<Stmt>>, Box<ariadne::Report<'static>>> {
    Resolver::new()
        .resolve_stmts(program)
        .map_err(|err| Box::new(error_report(&err, ariadne_config)))
}

pub fn resolve_expr(
    expr: &Spanned<ast::Expr>,
    ariadne_config: &ariadne::Config,
) -> Result<Spanned<Expr>, Box<ariadne::Report<'static>>> {
    Resolver::new()
        .resolve_expr(expr)
        .map_err(|err| Box::new(error_report(&err, ariadne_config)))
}

fn error_report<'a>(
    err: &'a Error,
    ariadne_config: &'a ariadne::Config,
) -> ariadne::Report<'static> {
    ariadne::Report::build(ariadne::ReportKind::Error, err.span.range().clone())
        .with_config(ariadne_config.with_index_type(ariadne::IndexType::Byte))
        .with_label(
            ariadne::Label::new(err.span.range().clone())
                .with_message(err.message.clone())
                .with_color(ariadne::Color::Red),
        )
        .finish()
}

impl Resolver {
    fn new() -> Self {
        Self {
            environments: vec![Environment::new_program()],
        }
    }

    fn resolve_stmts(
        &mut self,
        program: &Vec<Spanned<ast::Stmt>>,
    ) -> Result<Vec<Spanned<Stmt>>, Error> {
        let mut stmts = Vec::new();
        for stmt in program {
            let stmt = self.resolve_stmt(stmt)?;
            stmts.push(stmt);
        }
        Ok(stmts)
    }

    fn resolve_stmt(&mut self, stmt: &Spanned<ast::Stmt>) -> Result<Spanned<Stmt>, Error> {
        Ok(stmt.s.spanned(match &stmt.v {
            ast::Stmt::Expr(expr) => Stmt::Expr(self.resolve_expr(expr)?),
            ast::Stmt::Print(expr) => Stmt::Print(self.resolve_expr(expr)?),
            ast::Stmt::VarDecl(ident, expr) => {
                Stmt::VarDecl(self.declare(ident)?, self.resolve_expr(expr)?)
            }
            ast::Stmt::Block(ast_stmts) => {
                let (popped, stmts) = self.scoped(|resolver| {
                    let mut stmts = Vec::new();
                    for stmt in ast_stmts.iter() {
                        stmts.push(resolver.resolve_stmt(stmt)?);
                    }
                    Ok(stmts)
                })?;

                Stmt::Block(stmts, popped)
            }
            ast::Stmt::If { cond, then, else_ } => Stmt::If {
                cond: self.resolve_expr(cond)?,
                then: Box::new(self.resolve_stmt(then)?),
                else_: match else_ {
                    Some(else_) => Some(Box::new(self.resolve_stmt(else_)?)),
                    None => None,
                },
            },
            ast::Stmt::While { cond, body } => Stmt::While {
                cond: self.resolve_expr(cond)?,
                body: Box::new(self.resolve_stmt(body)?),
            },
            ast::Stmt::For {
                initializer,
                condition,
                increment,
                body,
            } => Stmt::For {
                initializer: match initializer {
                    Some(initializer) => Some(Box::new(self.resolve_stmt(initializer)?)),
                    None => None,
                },
                condition: match condition {
                    Some(condition) => Some(self.resolve_expr(condition)?),
                    None => None,
                },
                increment: match increment {
                    Some(increment) => Some(self.resolve_expr(increment)?),
                    None => None,
                },
                body: Box::new(self.resolve_stmt(body)?),
            },
            ast::Stmt::Break => Stmt::Break,
            ast::Stmt::Continue => Stmt::Continue,
            ast::Stmt::FunDecl(ident, parameters, stmts) => {
                let ident = self.declare(ident)?;

                self.in_closure(|resolver| {
                    let mut params = Vec::new();
                    for parameter in parameters {
                        params.push(resolver.declare(parameter)?);
                    }

                    Ok(Stmt::FunDecl(
                        ident,
                        parameters.len(),
                        resolver.resolve_stmts(stmts)?,
                    ))
                })?
            }
            ast::Stmt::Return(expr) => Stmt::Return(self.resolve_expr(expr)?),
        }))
    }

    fn resolve_expr(&mut self, expr: &Spanned<ast::Expr>) -> Result<Spanned<Expr>, Error> {
        Ok(expr.s.spanned(match &expr.v {
            ast::Expr::Number(n) => Expr::Number(*n),
            ast::Expr::String(s) => Expr::String(s.clone()),
            ast::Expr::Boolean(b) => Expr::Boolean(*b),
            ast::Expr::Nil => Expr::Nil,
            ast::Expr::Not(expr) => Expr::Not(Box::new(self.resolve_expr(expr)?)),
            ast::Expr::Neg(expr) => Expr::Neg(Box::new(self.resolve_expr(expr)?)),
            ast::Expr::Binary(lhs, binop, rhs) => Expr::Binary(
                Box::new(self.resolve_expr(lhs)?),
                *binop,
                Box::new(self.resolve_expr(rhs)?),
            ),
            ast::Expr::Assign(ident, expr) => {
                Expr::Assign(self.resolve(ident)?, Box::new(self.resolve_expr(expr)?))
            }
            ast::Expr::Var(ident) => Expr::Var(self.resolve(ident)?),
            ast::Expr::Call(f, ast_args) => Expr::Call(Box::new(self.resolve_expr(f)?), {
                let mut args = Vec::new();
                for arg in ast_args {
                    args.push(self.resolve_expr(arg)?);
                }
                args
            }),
            ast::Expr::Fun(ast_params, ast_stmts) => self.in_closure(|resolver| {
                let mut params = Vec::new();
                for param in ast_params {
                    params.push(resolver.declare(param)?);
                }
                let mut stmts = Vec::new();
                for stmt in ast_stmts {
                    stmts.push(resolver.resolve_stmt(stmt)?);
                }
                Ok(Expr::Fun(params, stmts))
            })?,
        }))
    }

    fn declare(&mut self, name: &Spanned<Rc<String>>) -> Result<Ident, Error> {
        if let Some(scope) = self.environment_mut().scopes.last_mut() {
            if scope.variables.contains_key(name.v.as_str()) {
                Err(Error {
                    message: format!(
                        "Variable '{}' has already been declared in this scope",
                        name.v.as_str()
                    ),
                    span: name.s.clone(),
                })
            } else {
                let d = scope.depth;
                scope.depth += 1;

                scope.variables.insert(name.v.to_string(), d);

                Ok(Ident::Local(d))
            }
        } else {
            Ok(Ident::Global(name.v.clone()))
        }
    }

    fn resolve(&self, name: &Spanned<Rc<String>>) -> Result<Ident, Error> {
        for scope in self.environment().scopes.iter().rev() {
            if let Some(d) = scope.variables.get(name.v.as_str()) {
                return Ok(Ident::Local(*d));
            }
        }

        Ok(Ident::Global(name.v.clone()))
    }

    fn enter_scope(&mut self) {
        let last_depth = self.environment().scopes.last().map_or(0, |s| s.depth);
        self.environment_mut().scopes.push(Scope {
            variables: HashMap::new(),
            depth: last_depth,
        });
    }

    fn exit_scope(&mut self) -> usize {
        self.environment_mut().scopes.pop().unwrap();

        self.environment().scopes.last().map_or(0, |s| s.depth)
    }

    fn environment(&self) -> &Environment {
        self.environments.last().unwrap()
    }

    fn environment_mut(&mut self) -> &mut Environment {
        self.environments.last_mut().unwrap()
    }

    fn scoped<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<T, Error>,
    ) -> Result<(usize, T), Error> {
        self.enter_scope();
        let res = f(self);
        let popped = self.exit_scope();
        res.map(|v| (popped, v))
    }

    fn enter_closure(&mut self) {
        // Create environment and scope within the environment
        self.environments.push(Environment::new_closure());
    }

    fn exit_closure(&mut self) {
        self.environments.pop().unwrap();
    }

    fn in_closure<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T, Error>) -> Result<T, Error> {
        self.enter_closure();
        let res = f(self);
        self.exit_closure();
        res
    }
}

#[derive(Debug, Clone)]
pub enum Ident {
    Global(Rc<String>),
    Local(usize),
}

pub use ast::Binop;

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    String(Rc<String>),
    Boolean(bool),
    Nil,
    Not(Box<Spanned<Expr>>),
    Neg(Box<Spanned<Expr>>),
    Binary(Box<Spanned<Expr>>, Binop, Box<Spanned<Expr>>),
    Assign(Ident, Box<Spanned<Expr>>),
    Var(Ident),
    Call(Box<Spanned<Expr>>, Vec<Spanned<Expr>>),
    Fun(Vec<Ident>, Vec<Spanned<Stmt>>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Spanned<Expr>),
    Print(Spanned<Expr>),
    VarDecl(Ident, Spanned<Expr>),
    /// The statements in the block, and the amount of variables declared before the block. Use to
    /// pop remaining declarations.
    Block(Vec<Spanned<Stmt>>, usize),
    If {
        cond: Spanned<Expr>,
        then: Box<Spanned<Stmt>>,
        else_: Option<Box<Spanned<Stmt>>>,
    },
    While {
        cond: Spanned<Expr>,
        body: Box<Spanned<Stmt>>,
    },
    For {
        initializer: Option<Box<Spanned<Stmt>>>,
        condition: Option<Spanned<Expr>>,
        increment: Option<Spanned<Expr>>,
        body: Box<Spanned<Stmt>>,
    },
    Break,
    Continue,
    FunDecl(Ident, usize, Vec<Spanned<Stmt>>),
    Return(Spanned<Expr>),
}
