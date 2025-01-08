mod interpret;
mod native;
pub mod value;

use interpret::{err, ControlFlow, InterpreterCtx};
use value::Value;

use qrlox_syntax::ast::{Expr, Span, Spanned, Stmt};

pub struct Error {
    pub message: String,
    pub span: Span,
}

pub struct Interpreter {
    ctx: InterpreterCtx,
}

impl Interpreter {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            ctx: InterpreterCtx::new(),
        }
    }

    pub fn exec_stmt<'a, 'b>(
        &'a mut self,
        stmt: &'a Spanned<Stmt<'a>>,
        ariadne_config: &'a ariadne::Config,
    ) -> Result<(), Box<ariadne::Report<'b>>> {
        self.ctx
            .exec_stmt(stmt)
            .map_err(|e| match e {
                ControlFlow::Break => err!(stmt.s, "Tried to break outside of a loop."),
                ControlFlow::Continue => err!(stmt.s, "Tried to continue outside of a loop."),
                ControlFlow::Error(error) => error,
            })
            .map_err(|err| Box::new(interpreter_error_report(&err, ariadne_config)))
    }

    pub fn exec_program<'a, 'b>(
        &'a mut self,
        stmts: &'a Vec<Spanned<Stmt<'a>>>,
        ariadne_config: &'a ariadne::Config,
    ) -> Result<(), Box<ariadne::Report<'b>>> {
        for stmt in stmts {
            self.exec_stmt(stmt, ariadne_config)?;
        }
        Ok(())
    }

    pub fn eval_expr<'a, 'b>(
        &'a mut self,
        expr: &'a Spanned<Expr<'a>>,
        ariadne_config: &'a ariadne::Config,
    ) -> Result<Value, Box<ariadne::Report<'b>>> {
        self.ctx
            .eval_expr(expr)
            .map_err(|e| match e {
                ControlFlow::Break => unreachable!("Cannot break in an expression."),
                ControlFlow::Continue => unreachable!("Cannot continue in an expression."),
                ControlFlow::Error(error) => error,
            })
            .map_err(|err| Box::new(interpreter_error_report(&err, ariadne_config)))
    }
}

pub fn interpreter_error_report<'a, 'b>(
    err: &'a Error,
    ariadne_config: &'a ariadne::Config,
) -> ariadne::Report<'b> {
    ariadne::Report::build(ariadne::ReportKind::Error, err.span.range().clone())
        .with_config(ariadne_config.with_index_type(ariadne::IndexType::Byte))
        .with_label(
            ariadne::Label::new(err.span.range().clone())
                .with_message(err.message.clone())
                .with_color(ariadne::Color::Red),
        )
        .finish()
}
