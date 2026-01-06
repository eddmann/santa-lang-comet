use super::doc::Doc;
use crate::parser::ast::{Expression, ExpressionKind, Infix, MatchCase, Prefix, Program, Statement, StatementKind};

/// Standard indentation level in spaces.
/// All nested blocks use this consistent indent for visual hierarchy.
const INDENT_SIZE: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Lowest = 0,
    AndOr,
    Equals,
    LessGreater,
    Composition,
    Sum,
    Product,
}

pub fn build_program(program: &Program) -> Doc {
    if program.statements.is_empty() {
        return Doc::Nil;
    }

    let mut result = Vec::new();
    for (i, stmt) in program.statements.iter().enumerate() {
        if i > 0 {
            // Always blank line between top-level statements
            result.push(Doc::HardLine);
            result.push(Doc::HardLine);
        }
        result.push(build_statement(stmt, true));

        // Emit trailing comment if present
        if let Some(comment) = &stmt.trailing_comment {
            result.push(Doc::text(format!(" {}", comment)));
        }
    }
    result.push(Doc::HardLine);

    Doc::concat(result)
}

fn build_statements(statements: &[Statement]) -> Doc {
    if statements.is_empty() {
        return Doc::Nil;
    }

    let len = statements.len();
    let mut result = Vec::new();

    let semicolon_index = if len >= 2 {
        let has_implicit_return = matches!(
            &statements[len - 1].kind,
            StatementKind::Expression(expr) if !matches!(expr.kind, ExpressionKind::Let { .. } | ExpressionKind::MutableLet { .. })
        );
        if has_implicit_return {
            statements[..len - 1]
                .iter()
                .rposition(|s| !matches!(s.kind, StatementKind::Comment(_)))
        } else {
            None
        }
    } else {
        None
    };

    for (i, stmt) in statements.iter().enumerate() {
        if i > 0 {
            // Preserve user blank lines from source, or add for implicit returns
            let needs_blank = stmt.preceded_by_blank_line
                || match &stmt.kind {
                    StatementKind::Expression(expr) if i == len - 1 && len > 1 => !matches!(
                        expr.kind,
                        ExpressionKind::Let { .. } | ExpressionKind::MutableLet { .. }
                    ),
                    StatementKind::Return(expr) if len > 1 => is_multiline_expression(expr),
                    _ => false,
                };

            if needs_blank {
                result.push(Doc::BlankLine);
                result.push(Doc::HardLine);
            } else {
                result.push(Doc::HardLine);
            }
        }
        result.push(build_statement(stmt, false));

        if semicolon_index == Some(i) {
            result.push(Doc::text(";"));
        }

        // Emit trailing comment if present
        if let Some(comment) = &stmt.trailing_comment {
            result.push(Doc::text(format!(" {}", comment)));
        }
    }

    Doc::concat(result)
}

fn build_statement(stmt: &Statement, is_top_level: bool) -> Doc {
    match &stmt.kind {
        StatementKind::Return(expr) => Doc::concat(vec![Doc::text("return "), build_expression(expr)]),

        StatementKind::Break(expr) => Doc::concat(vec![Doc::text("break "), build_expression(expr)]),

        StatementKind::Comment(text) => Doc::text(text.as_str()),

        StatementKind::Section { name, body, attributes } => {
            let mut parts = Vec::new();

            for attr in attributes {
                parts.push(Doc::text(format!("@{}", attr.name)));
                parts.push(Doc::HardLine);
            }

            parts.push(Doc::text(format!("{}: ", name)));

            let always_braces = is_top_level && (name == "part_one" || name == "part_two");

            if !always_braces && body.statements.len() == 1 {
                if let StatementKind::Expression(expr) = &body.statements[0].kind {
                    if !contains_block_lambda(expr) {
                        parts.push(build_expression(expr));
                        return Doc::concat(parts);
                    }
                }
            }

            parts.push(Doc::text("{"));
            parts.push(Doc::nest(
                INDENT_SIZE,
                Doc::concat(vec![Doc::HardLine, build_statements(&body.statements)]),
            ));
            parts.push(Doc::HardLine);
            parts.push(Doc::text("}"));

            Doc::concat(parts)
        }

        StatementKind::Expression(expr) => build_expression(expr),

        StatementKind::Block(statements) => {
            if statements.is_empty() {
                return Doc::text("{}");
            }

            Doc::concat(vec![
                Doc::text("{"),
                Doc::nest(
                    INDENT_SIZE,
                    Doc::concat(vec![Doc::HardLine, build_statements(statements)]),
                ),
                Doc::HardLine,
                Doc::text("}"),
            ])
        }
    }
}

fn build_expression(expr: &Expression) -> Doc {
    match &expr.kind {
        // Literals
        ExpressionKind::Integer(value) => Doc::text(value),
        ExpressionKind::Decimal(value) => Doc::text(value),
        ExpressionKind::String(value) => build_string(value),
        ExpressionKind::Boolean(value) => Doc::text(if *value { "true" } else { "false" }),
        ExpressionKind::Nil => Doc::text("nil"),
        ExpressionKind::Placeholder => Doc::text("_"),

        // Identifiers
        ExpressionKind::Identifier(name) => Doc::text(name),
        ExpressionKind::RestIdentifier(name) => Doc::concat(vec![Doc::text(".."), Doc::text(name)]),

        // Bindings
        ExpressionKind::Let { name, value } => build_let(name, value, false),
        ExpressionKind::MutableLet { name, value } => build_let(name, value, true),
        ExpressionKind::Assign { name, value } => build_assign(name, value),

        // Collections
        ExpressionKind::List(elements) => build_collection("[", elements, "]"),
        ExpressionKind::Set(elements) => build_collection("{", elements, "}"),
        ExpressionKind::Dictionary(entries) => build_dictionary(entries),

        // Ranges
        ExpressionKind::InclusiveRange { from, to } => build_range(from, to, "..="),
        ExpressionKind::ExclusiveRange { from, until } => build_range(from, until, ".."),
        ExpressionKind::UnboundedRange { from } => Doc::concat(vec![build_expression(from), Doc::text("..")]),

        // Functions
        ExpressionKind::Function { parameters, body } => build_lambda(parameters, body),
        ExpressionKind::Call { function, arguments } => build_call(function, arguments),

        // Operators
        ExpressionKind::Prefix { operator, right } => build_prefix_expr(operator, right),
        ExpressionKind::Infix { operator, left, right } => build_infix_expr(operator, left, right),

        // Control flow
        ExpressionKind::If {
            condition,
            consequence,
            alternative,
        } => build_if(condition, consequence, alternative),
        ExpressionKind::Match { subject, cases } => build_match(subject, cases),

        // Functional operations
        ExpressionKind::FunctionThread { initial, functions } => build_chain(initial, functions, "|>"),
        ExpressionKind::FunctionComposition(functions) => build_composition(functions),

        // Other
        ExpressionKind::Index { left, index } => build_index(left, index),
        ExpressionKind::Spread(inner) => Doc::concat(vec![Doc::text(".."), build_expression(inner)]),
        ExpressionKind::IdentifierListPattern(elements) => build_pattern(elements),
        ExpressionKind::ListMatchPattern(elements) => build_pattern(elements),
        ExpressionKind::IdentifierDictionaryPattern(elements) => build_dictionary_pattern(elements),
        ExpressionKind::DictionaryMatchPattern(elements) => build_dictionary_pattern(elements),
        ExpressionKind::DictionaryEntryPattern { key, value } => {
            Doc::concat(vec![build_expression(key), Doc::text(": "), build_expression(value)])
        }
    }
}

fn build_assign(name: &Expression, value: &Expression) -> Doc {
    Doc::concat(vec![build_expression(name), Doc::text(" = "), build_expression(value)])
}

fn build_call(function: &Expression, arguments: &[Expression]) -> Doc {
    if arguments.is_empty() {
        return Doc::concat(vec![build_expression(function), Doc::text("()")]);
    }

    let Some(trailing) = extract_trailing_closure(arguments) else {
        let args: Vec<Doc> = arguments.iter().map(build_expression).collect();
        return Doc::concat(vec![build_expression(function), Doc::bracketed("(", args, ")", false)]);
    };

    let func = build_expression(function);
    let block_lambda = build_lambda_with_block(trailing.parameters, trailing.body);

    let build_trailing_doc = |func: Doc, lambda: Doc| -> Doc {
        if trailing.is_only_argument {
            Doc::concat(vec![func, Doc::text(" "), lambda])
        } else {
            let other_args: Vec<Doc> = arguments[..arguments.len() - 1].iter().map(build_expression).collect();
            Doc::concat(vec![
                func,
                Doc::bracketed("(", other_args, ")", false),
                Doc::text(" "),
                lambda,
            ])
        }
    };

    // Multi-statement lambdas always use trailing syntax
    if trailing.is_multi_statement {
        return build_trailing_doc(func, block_lambda);
    }

    // Single-statement: use trailing with block if line would exceed width
    let inline_lambda = build_lambda(trailing.parameters, trailing.body);
    let inline_doc = if trailing.is_only_argument {
        Doc::concat(vec![func.clone(), Doc::text("("), inline_lambda, Doc::text(")")])
    } else {
        let all_args: Vec<Doc> = arguments.iter().map(build_expression).collect();
        Doc::concat(vec![func.clone(), Doc::bracketed("(", all_args, ")", false)])
    };

    let trailing_doc = build_trailing_doc(func, block_lambda);
    Doc::group(Doc::if_break(trailing_doc, inline_doc))
}

fn build_composition(functions: &[Expression]) -> Doc {
    if functions.is_empty() {
        return Doc::Nil;
    }

    // Line-width based formatting: let the printer decide when to break
    // based on the 100-character line width limit
    let docs: Vec<Doc> = functions.iter().map(build_expression).collect();

    let rest: Vec<Doc> = docs[1..]
        .iter()
        .map(|d| Doc::concat(vec![Doc::line(), Doc::text(">> "), d.clone()]))
        .collect();

    Doc::group(Doc::concat(vec![
        docs[0].clone(),
        Doc::nest(INDENT_SIZE, Doc::concat(rest)),
    ]))
}

fn build_dictionary(entries: &[(Expression, Expression)]) -> Doc {
    if entries.is_empty() {
        return Doc::text("#{}");
    }

    let docs: Vec<Doc> = entries
        .iter()
        .map(|(k, v)| match (&k.kind, &v.kind) {
            (ExpressionKind::String(key), ExpressionKind::Identifier(name)) if key == name => build_expression(v),
            _ => Doc::concat(vec![build_expression(k), Doc::text(": "), build_expression(v)]),
        })
        .collect();

    Doc::concat(vec![Doc::text("#"), Doc::bracketed("{", docs, "}", false)])
}

fn build_if(condition: &Expression, consequence: &Statement, alternative: &Option<Box<Statement>>) -> Doc {
    let inline_doc = build_inline_if(condition, consequence, alternative);
    let multiline_doc = build_multiline_if(condition, consequence, alternative);
    Doc::group(Doc::if_break(multiline_doc, inline_doc))
}

fn build_index(left: &Expression, index: &Expression) -> Doc {
    Doc::concat(vec![
        build_expression(left),
        Doc::text("["),
        build_expression(index),
        Doc::text("]"),
    ])
}

fn build_infix_expr(operator: &Infix, left: &Expression, right: &Expression) -> Doc {
    let op_prec = infix_precedence(operator);
    let left_doc = build_left_expr_with_parens(left, op_prec);
    let right_doc = build_right_expr_with_parens(right, op_prec);

    Doc::group(Doc::concat(vec![
        left_doc,
        Doc::text(" "),
        build_infix(operator),
        Doc::text(" "),
        right_doc,
    ]))
}

fn build_let(name: &Expression, value: &Expression, is_mutable: bool) -> Doc {
    let prefix = if is_mutable { "let mut " } else { "let " };
    Doc::concat(vec![
        Doc::text(prefix),
        build_expression(name),
        Doc::text(" = "),
        build_expression(value),
    ])
}

fn build_match(subject: &Expression, cases: &[MatchCase]) -> Doc {
    let case_docs: Vec<Doc> = cases.iter().map(build_match_case).collect();

    Doc::concat(vec![
        Doc::text("match "),
        build_expression(subject),
        Doc::text(" {"),
        Doc::nest(
            INDENT_SIZE,
            Doc::concat(vec![Doc::HardLine, Doc::join(case_docs, Doc::HardLine)]),
        ),
        Doc::HardLine,
        Doc::text("}"),
    ])
}

fn build_pattern(elements: &[Expression]) -> Doc {
    let docs: Vec<Doc> = elements.iter().map(build_expression).collect();
    Doc::concat(vec![Doc::text("["), Doc::join(docs, Doc::text(", ")), Doc::text("]")])
}

fn build_dictionary_pattern(elements: &[Expression]) -> Doc {
    let docs: Vec<Doc> = elements.iter().map(build_expression).collect();
    Doc::concat(vec![Doc::text("#{"), Doc::join(docs, Doc::text(", ")), Doc::text("}")])
}

fn build_prefix_expr(operator: &Prefix, right: &Expression) -> Doc {
    let right_doc = build_expression(right);
    let needs_parens = matches!(
        right.kind,
        ExpressionKind::Infix { .. } | ExpressionKind::FunctionThread { .. } | ExpressionKind::FunctionComposition(_)
    );
    if needs_parens {
        Doc::concat(vec![build_prefix(operator), Doc::text("("), right_doc, Doc::text(")")])
    } else {
        Doc::concat(vec![build_prefix(operator), right_doc])
    }
}

fn build_range(from: &Expression, to: &Expression, op: &str) -> Doc {
    Doc::concat(vec![build_expression(from), Doc::text(op), build_expression(to)])
}

fn build_string(value: &str) -> Doc {
    let escaped = escape_string(value);
    Doc::text(format!("\"{}\"", escaped))
}

/// Extracts the last argument if it's a lambda function
struct TrailingClosure<'a> {
    parameters: &'a [Expression],
    body: &'a Statement,
    is_only_argument: bool,
    is_multi_statement: bool,
}

fn extract_trailing_closure(arguments: &[Expression]) -> Option<TrailingClosure<'_>> {
    let last_arg = arguments.last()?;

    let (parameters, body) = match &last_arg.kind {
        ExpressionKind::Function { parameters, body } => (parameters.as_slice(), body.as_ref()),
        _ => return None,
    };

    let is_multi_statement = match &body.kind {
        StatementKind::Block(stmts) => stmts.len() > 1,
        _ => false,
    };

    Some(TrailingClosure {
        parameters,
        body,
        is_only_argument: arguments.len() == 1,
        is_multi_statement,
    })
}

fn build_block_body(stmts: &[Statement]) -> Doc {
    if stmts.is_empty() {
        Doc::text("{}")
    } else {
        Doc::concat(vec![
            Doc::text("{"),
            Doc::nest(INDENT_SIZE, Doc::concat(vec![Doc::HardLine, build_statements(stmts)])),
            Doc::HardLine,
            Doc::text("}"),
        ])
    }
}

fn build_block_statement(stmt: &Statement) -> Doc {
    match &stmt.kind {
        StatementKind::Block(_) => build_statement(stmt, false),
        _ => Doc::concat(vec![
            Doc::text("{"),
            Doc::nest(
                INDENT_SIZE,
                Doc::concat(vec![Doc::HardLine, build_statement(stmt, false)]),
            ),
            Doc::HardLine,
            Doc::text("}"),
        ]),
    }
}

fn build_chain(initial: &Expression, functions: &[Expression], op: &str) -> Doc {
    // Special case: single-pipe chain with trailing block lambda
    // Format as `initial |> func |params| { block }` without group wrapping
    // to prevent the block's HardLines from forcing the pipe to break
    if functions.len() == 1 {
        if let Some(call_doc) = build_call_for_chain(&functions[0]) {
            return Doc::concat(vec![
                build_expression(initial),
                Doc::text(format!(" {} ", op)),
                call_doc,
            ]);
        }
    }

    let force_break = functions.len() > 1;

    let chain: Vec<Doc> = functions
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let is_last = i == functions.len() - 1;
            // Lambdas that aren't the last element need block braces to prevent
            // the next pipe from being parsed as part of the lambda body
            let f_doc = match &f.kind {
                ExpressionKind::Function { parameters, body } if !is_last => build_lambda_with_block(parameters, body),
                _ => build_expression(f),
            };
            Doc::concat(vec![
                if force_break { Doc::HardLine } else { Doc::line() },
                Doc::text(format!("{} ", op)),
                f_doc,
            ])
        })
        .collect();

    let doc = Doc::concat(vec![
        build_expression(initial),
        Doc::nest(INDENT_SIZE, Doc::concat(chain)),
    ]);

    if force_break { doc } else { Doc::group(doc) }
}

/// Build a call expression for use in a single-element pipe chain.
/// Returns Some(Doc) if the call has a trailing lambda that will use block syntax,
/// allowing the pipe to stay on one line while the block breaks.
/// Returns None for regular calls or short inline lambdas (use normal build_expression).
fn build_call_for_chain(expr: &Expression) -> Option<Doc> {
    let ExpressionKind::Call { function, arguments } = &expr.kind else {
        return None;
    };

    let trailing = extract_trailing_closure(arguments)?;

    // Multi-statement lambdas always use trailing block syntax
    if trailing.is_multi_statement {
        let func = build_expression(function);
        let block_lambda = build_lambda_with_block(trailing.parameters, trailing.body);

        return Some(if trailing.is_only_argument {
            Doc::concat(vec![func, Doc::text(" "), block_lambda])
        } else {
            let other_args: Vec<Doc> = arguments[..arguments.len() - 1].iter().map(build_expression).collect();
            Doc::concat(vec![
                func,
                Doc::bracketed("(", other_args, ")", false),
                Doc::text(" "),
                block_lambda,
            ])
        });
    }

    // For single-statement lambdas, check if inline form would be short enough
    // If not, use trailing block syntax to keep pipe on one line
    let func = build_expression(function);
    let inline_lambda = build_lambda(trailing.parameters, trailing.body);
    let block_lambda = build_lambda_with_block(trailing.parameters, trailing.body);

    let inline_doc = if trailing.is_only_argument {
        Doc::concat(vec![func.clone(), Doc::text("("), inline_lambda, Doc::text(")")])
    } else {
        let all_args: Vec<Doc> = arguments.iter().map(build_expression).collect();
        Doc::concat(vec![func.clone(), Doc::bracketed("(", all_args, ")", false)])
    };

    let trailing_doc = if trailing.is_only_argument {
        Doc::concat(vec![func, Doc::text(" "), block_lambda])
    } else {
        let other_args: Vec<Doc> = arguments[..arguments.len() - 1].iter().map(build_expression).collect();
        Doc::concat(vec![
            func,
            Doc::bracketed("(", other_args, ")", false),
            Doc::text(" "),
            block_lambda,
        ])
    };

    // Return a group that chooses between inline and trailing based on width
    Some(Doc::group(Doc::if_break(trailing_doc, inline_doc)))
}

fn build_collection(open: &str, elements: &[Expression], close: &str) -> Doc {
    if elements.is_empty() {
        return Doc::concat(vec![Doc::text(open), Doc::text(close)]);
    }

    let docs: Vec<Doc> = elements.iter().map(build_expression).collect();

    Doc::bracketed(open, docs, close, false)
}

fn build_infix(op: &Infix) -> Doc {
    match op {
        Infix::Plus => Doc::text("+"),
        Infix::Minus => Doc::text("-"),
        Infix::Asterisk => Doc::text("*"),
        Infix::Slash => Doc::text("/"),
        Infix::Modulo => Doc::text("%"),
        Infix::Equal => Doc::text("=="),
        Infix::NotEqual => Doc::text("!="),
        Infix::LessThan => Doc::text("<"),
        Infix::LessThanEqual => Doc::text("<="),
        Infix::GreaterThan => Doc::text(">"),
        Infix::GreaterThanEqual => Doc::text(">="),
        Infix::Or => Doc::text("||"),
        Infix::And => Doc::text("&&"),
        Infix::Call(ident) => Doc::concat(vec![Doc::text("`"), build_expression(ident), Doc::text("`")]),
    }
}

fn build_inline_body(stmt: &Statement) -> Doc {
    match &stmt.kind {
        StatementKind::Expression(expr) => build_expression(expr),
        StatementKind::Block(stmts) if stmts.len() == 1 => {
            if let StatementKind::Expression(expr) = &stmts[0].kind {
                build_expression(expr)
            } else {
                build_block_statement(stmt)
            }
        }
        _ => build_block_statement(stmt),
    }
}

fn build_inline_if(condition: &Expression, consequence: &Statement, alternative: &Option<Box<Statement>>) -> Doc {
    let mut parts = vec![
        Doc::text("if "),
        build_expression(condition),
        Doc::text(" { "),
        build_inline_body(consequence),
        Doc::text(" }"),
    ];

    if let Some(alt) = alternative {
        parts.push(Doc::text(" else { "));
        parts.push(build_inline_body(alt));
        parts.push(Doc::text(" }"));
    }

    Doc::concat(parts)
}

fn build_lambda(parameters: &[Expression], body: &Statement) -> Doc {
    let params: Vec<Doc> = parameters.iter().map(build_expression).collect();
    let params_doc = Doc::join(params, Doc::text(", "));

    let body_doc = match &body.kind {
        StatementKind::Block(stmts) if stmts.len() == 1 => {
            // Single statement block - check if it's a simple expression
            match &stmts[0].kind {
                StatementKind::Expression(expr) => {
                    // Don't unwrap if:
                    // 1. Expression is a set or dictionary - braces would be confused with lambda body
                    // 2. Expression contains pipes or compositions - they would bind to the lambda definition
                    //    instead of being part of the lambda body
                    match &expr.kind {
                        ExpressionKind::Set(_) | ExpressionKind::Dictionary(_) => build_block_body(stmts),
                        _ if has_pipe_or_composition(expr) => build_block_body(stmts),
                        _ => build_expression(expr),
                    }
                }
                _ => build_block_body(stmts),
            }
        }
        StatementKind::Block(stmts) => build_block_body(stmts),
        StatementKind::Expression(expr) => build_expression(expr),
        _ => build_statement(body, false),
    };

    Doc::concat(vec![Doc::text("|"), params_doc, Doc::text("| "), body_doc])
}

fn build_lambda_with_block(parameters: &[Expression], body: &Statement) -> Doc {
    let params: Vec<Doc> = parameters.iter().map(build_expression).collect();
    let params_doc = Doc::join(params, Doc::text(", "));

    let body_doc = match &body.kind {
        StatementKind::Block(stmts) => build_block_body(stmts),
        _ => build_block_statement(body),
    };

    Doc::concat(vec![Doc::text("|"), params_doc, Doc::text("| "), body_doc])
}

fn build_left_expr_with_parens(expr: &Expression, parent_prec: Precedence) -> Doc {
    let expr_prec = expression_precedence(expr);
    let doc = build_expression(expr);

    if expr_prec < parent_prec && expr_prec != Precedence::Lowest {
        Doc::concat(vec![Doc::text("("), doc, Doc::text(")")])
    } else {
        doc
    }
}

fn build_match_case(case: &MatchCase) -> Doc {
    let mut parts = vec![build_expression(&case.pattern)];

    if let Some(guard) = &case.guard {
        parts.push(Doc::text(" if "));
        parts.push(build_expression(guard));
    }

    if is_simple_body(&case.consequence) {
        parts.push(Doc::text(" { "));
        parts.push(build_inline_body(&case.consequence));
        parts.push(Doc::text(" }"));
    } else {
        parts.push(Doc::text(" "));
        parts.push(build_block_statement(&case.consequence));
    }

    if let Some(comment) = &case.trailing_comment {
        parts.push(Doc::text(format!(" {}", comment)));
    }

    Doc::concat(parts)
}

fn build_multiline_if(condition: &Expression, consequence: &Statement, alternative: &Option<Box<Statement>>) -> Doc {
    let mut parts = vec![
        Doc::text("if "),
        build_expression(condition),
        Doc::text(" "),
        build_block_statement(consequence),
    ];

    if let Some(alt) = alternative {
        parts.push(Doc::text(" else "));
        parts.push(build_block_statement(alt));
    }

    Doc::concat(parts)
}

fn build_prefix(op: &Prefix) -> Doc {
    match op {
        Prefix::Bang => Doc::text("!"),
        Prefix::Minus => Doc::text("-"),
    }
}

fn build_right_expr_with_parens(expr: &Expression, parent_prec: Precedence) -> Doc {
    let expr_prec = expression_precedence(expr);
    let doc = build_expression(expr);

    // For left-associative operators, any same-precedence expression on the right
    // needs parentheses to preserve the original grouping.
    // E.g., `a + (b + c)` must keep parens, otherwise it becomes `(a + b) + c`
    let needs_parens = expr_prec != Precedence::Lowest && expr_prec <= parent_prec;

    if needs_parens {
        Doc::concat(vec![Doc::text("("), doc, Doc::text(")")])
    } else {
        doc
    }
}

fn infix_precedence(op: &Infix) -> Precedence {
    match op {
        Infix::And | Infix::Or => Precedence::AndOr,
        Infix::Equal | Infix::NotEqual => Precedence::Equals,
        Infix::LessThan | Infix::LessThanEqual | Infix::GreaterThan | Infix::GreaterThanEqual => {
            Precedence::LessGreater
        }
        Infix::Plus | Infix::Minus => Precedence::Sum,
        Infix::Asterisk | Infix::Slash | Infix::Modulo | Infix::Call(_) => Precedence::Product,
    }
}

fn expression_precedence(expr: &Expression) -> Precedence {
    match &expr.kind {
        ExpressionKind::Infix { operator, .. } => infix_precedence(operator),
        ExpressionKind::FunctionThread { .. } | ExpressionKind::FunctionComposition(_) => Precedence::Composition,
        ExpressionKind::InclusiveRange { .. }
        | ExpressionKind::ExclusiveRange { .. }
        | ExpressionKind::UnboundedRange { .. } => Precedence::Composition,
        _ => Precedence::Lowest,
    }
}

fn has_pipe_or_composition(expr: &Expression) -> bool {
    matches!(
        &expr.kind,
        ExpressionKind::FunctionThread { .. } | ExpressionKind::FunctionComposition(_)
    )
}

fn is_simple_body(stmt: &Statement) -> bool {
    match &stmt.kind {
        StatementKind::Expression(expr) => !contains_block_lambda(expr),
        StatementKind::Block(stmts) if stmts.len() == 1 => {
            if let StatementKind::Expression(expr) = &stmts[0].kind {
                !contains_block_lambda(expr)
            } else {
                false
            }
        }
        _ => false,
    }
}

fn contains_block_lambda(expr: &Expression) -> bool {
    match &expr.kind {
        ExpressionKind::Function { body, .. } => match &body.kind {
            StatementKind::Block(stmts) => {
                stmts.len() > 1 || stmts.iter().any(|s| !matches!(s.kind, StatementKind::Expression(_)))
            }
            _ => false,
        },
        ExpressionKind::Call { function, arguments } => {
            contains_block_lambda(function) || arguments.iter().any(contains_block_lambda)
        }
        ExpressionKind::FunctionThread { initial, functions } => {
            contains_block_lambda(initial) || functions.iter().any(contains_block_lambda)
        }
        ExpressionKind::FunctionComposition(functions) => functions.iter().any(contains_block_lambda),
        ExpressionKind::Infix { left, right, .. } => contains_block_lambda(left) || contains_block_lambda(right),
        ExpressionKind::Prefix { right, .. } => contains_block_lambda(right),
        ExpressionKind::Index { left, index } => contains_block_lambda(left) || contains_block_lambda(index),
        ExpressionKind::List(elements) | ExpressionKind::Set(elements) => elements.iter().any(contains_block_lambda),
        ExpressionKind::Dictionary(entries) => entries
            .iter()
            .any(|(k, v)| contains_block_lambda(k) || contains_block_lambda(v)),
        ExpressionKind::If {
            condition,
            consequence,
            alternative,
        } => {
            contains_block_lambda(condition)
                || contains_block_lambda_in_stmt(consequence)
                || alternative.as_ref().is_some_and(|a| contains_block_lambda_in_stmt(a))
        }
        ExpressionKind::Match { .. } => true,
        _ => false,
    }
}

fn contains_block_lambda_in_stmt(stmt: &Statement) -> bool {
    match &stmt.kind {
        StatementKind::Expression(expr) => contains_block_lambda(expr),
        StatementKind::Block(stmts) => stmts.iter().any(contains_block_lambda_in_stmt),
        StatementKind::Return(expr) | StatementKind::Break(expr) => contains_block_lambda(expr),
        StatementKind::Section { body, .. } => body.statements.iter().any(contains_block_lambda_in_stmt),
        StatementKind::Comment(_) => false,
    }
}

fn is_multiline_expression(expr: &Expression) -> bool {
    match &expr.kind {
        ExpressionKind::FunctionThread { functions, .. } => functions.len() > 1,
        ExpressionKind::FunctionComposition(fns) => fns.len() > 1,
        ExpressionKind::Match { .. } => true,
        ExpressionKind::Function { body, .. } => {
            matches!(&body.kind, StatementKind::Block(stmts) if stmts.len() > 1)
        }
        _ => false,
    }
}

fn escape_string(s: &str) -> String {
    let newline_count = s.chars().filter(|&c| c == '\n').count();
    let is_multiline_content = newline_count > 3 || s.len() > 50;

    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' if !is_multiline_content => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            '\r' => result.push_str("\\r"),
            '\x08' => result.push_str("\\b"), // Backspace
            '\x0C' => result.push_str("\\f"), // Form feed
            _ => result.push(c),
        }
    }
    result
}
