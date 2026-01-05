// Allow collapsible_if as clippy suggests `let` chains which are unstable (Rust 1.87+)
#![allow(clippy::collapsible_if)]

mod evaluator;
mod formatter;
mod lexer;
mod parser;
mod runner;

pub use crate::evaluator::{Arguments, Environment, Evaluation, Evaluator, ExternalFnDef, Object, RuntimeErr};
pub use crate::formatter::{format, is_formatted};
pub use crate::lexer::{Lexer, Location, TokenKind};
pub use crate::parser::{Parser, ParserErr, ast::ExpressionKind};
pub use crate::runner::{AoCRunner, RunErr, RunEvaluation, RunResult, TestCase, Time};
