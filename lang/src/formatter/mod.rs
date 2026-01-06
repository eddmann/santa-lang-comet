//! Opinionated code formatter for santa-lang.
//!
//! This module implements a Wadler-style pretty printer that transforms
//! santa-lang source code into a canonical, standardized format.
//!
//! # Design Philosophy
//!
//! The formatter is **opinionated and deterministic**:
//! - No configuration options (like `gofmt`)
//! - Idempotent: `format(format(x)) == format(x)`
//! - Preserves semantic meaning while standardizing style
//!
//! # Style Decisions
//!
//! - **Line width**: 100 characters
//! - **Indentation**: 2 spaces
//! - **Pipe chains** (`|>`): Always multiline when 2+ functions
//! - **Composition** (`>>`): Line-width based (inline if fits, otherwise multiline)
//! - **Trailing closures**: Preserved for multi-statement lambdas
//! - **Operator spacing**: Always padded (`1 + 2`, not `1+2`)
//! - **Collections**: Inline if under line width, otherwise multiline
//!
//! # Architecture
//!
//! The formatter uses a three-phase design:
//! 1. **Parse**: Source code → AST (reuses existing lexer/parser)
//! 2. **Build**: AST → Document IR (`Doc` enum)
//! 3. **Print**: Document IR → Formatted string (with line-breaking)

mod builder;
mod doc;
mod printer;

#[cfg(test)]
mod tests;

use crate::lexer::Lexer;
use crate::parser::{Parser, ParserErr};

/// Formats santa-lang source code according to the opinionated style guide.
///
/// This function parses the source code, transforms it into an intermediate
/// document representation, and renders it with intelligent line-breaking.
///
/// # Errors
///
/// Returns `ParserErr` if the source code contains syntax errors.
/// The formatter cannot fix invalid code—it only reformats valid code.
///
/// # Examples
///
/// ```ignore
/// use lang::format;
///
/// let ugly = "let x=1+2";
/// let pretty = format(ugly).unwrap();
/// assert_eq!(pretty, "let x = 1 + 2\n");
///
/// // Formatting is idempotent
/// assert_eq!(format(&pretty).unwrap(), pretty);
/// ```
pub fn format(source: &str) -> Result<String, ParserErr> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let program = parser.parse()?;
    let doc = builder::build_program(&program);
    Ok(printer::print(&doc))
}

/// Checks if source code is already formatted according to the style guide.
///
/// This is equivalent to `format(source)? == source` but communicates intent
/// clearly. Useful for CI checks or editor integrations that want to warn
/// about unformatted code without actually reformatting.
///
/// # Errors
///
/// Returns `ParserErr` if the source code contains syntax errors.
///
/// # Examples
///
/// ```ignore
/// use lang::is_formatted;
///
/// assert!(is_formatted("let x = 1 + 2\n").unwrap());
/// assert!(!is_formatted("let x=1+2").unwrap());
/// ```
pub fn is_formatted(source: &str) -> Result<bool, ParserErr> {
    let formatted = format(source)?;
    Ok(formatted == source)
}
