pub mod ast;
pub mod cli;
pub mod interpreter;
pub mod lex;
pub mod parse;
pub mod repl;

pub const ARIADNE_CONFIG: ariadne::Config =
    ariadne::Config::new().with_index_type(ariadne::IndexType::Byte);
