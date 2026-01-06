# Comet Architecture

Comet is a tree-walking interpreter for santa-lang, implemented in Rust. This document describes the internal architecture, key design decisions, and implementation patterns.

## Overview

Comet follows a traditional interpreter pipeline:

```
Source Code --> Lexer --> Parser --> Evaluator --> Result
     |            |          |           |
   String      Tokens       AST      Object
```

The implementation prioritizes:

- **Correctness**: Full santa-lang specification compliance
- **Expressiveness**: Clean Rust code leveraging the type system
- **Portability**: Core `lang` crate shared across multiple runtimes

## Project Structure

```
santa-lang-comet/
├── lang/                    # Core language library
│   └── src/
│       ├── lexer/           # Tokenization
│       ├── parser/          # AST construction
│       ├── evaluator/       # Tree-walking interpreter
│       └── formatter/       # Code formatter
├── runtime/
│   ├── cli/                 # Command-line interface
│   ├── wasm/                # WebAssembly build
│   ├── lambda/              # AWS Lambda runtime
│   ├── jupyter/             # Jupyter kernel
│   └── php-ext/             # PHP extension
└── benchmarks/              # Performance benchmarks
```

## Lexer

The lexer (`lang/src/lexer/`) converts source text into tokens using a character-by-character scan.

### Design

- **Iterator-based**: Implements `Iterator<Item = Token>` for streaming tokenization
- **Position tracking**: Each token carries source location for error messages
- **Single-pass**: No backtracking, uses one-character lookahead
- **Trivia handling**: Tracks blank lines for formatter preservation

### Token Structure

```rust
pub struct Token {
    pub kind: TokenKind,
    pub source: Location,
    pub line: usize,
    pub preceded_by_blank_line: bool,
}

pub struct Location {
    pub start: usize,
    pub end: usize,
}
```

### Token Kinds

Tokens are represented as a flat enum with macro shortcuts for ergonomic matching:

```rust
#[repr(u8)]
pub enum TokenKind {
    // Literals
    Integer, Decimal, String, Identifier,

    // Operators
    Plus, Minus, Asterisk, Slash, Modulo,
    Equal, NotEqual, LessThan, GreaterThan,

    // Delimiters
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,

    // Keywords
    Let, If, Else, Match, Return, Break, ...
}

// Macro for cleaner token matching
match token.kind {
    T![+] => ...,    // TokenKind::Plus
    T![|>] => ...,   // TokenKind::PipeGreater
    T![LET] => ...,  // TokenKind::Let
}
```

## Parser

The parser (`lang/src/parser/`) builds an Abstract Syntax Tree using a Pratt (top-down operator precedence) parser.

### Pratt Parser

Pratt parsing elegantly handles operator precedence and associativity:

```rust
#[repr(u8)]
enum Precedence {
    Lowest = 0,
    AndOr,        // && ||
    Equals,       // == != =
    LessGreater,  // < <= > >=
    Composition,  // >> |> .. ..=
    Sum,          // + -
    Product,      // * / %
    Prefix,       // -x !x
    Call,         // f(x)
    Index,        // a[i]
}

fn parse_expression(&mut self, precedence: Precedence) -> RExpression {
    let mut left = self.parse_prefix_expression()?;

    while precedence < infix_binding_precedence(&self.current_token.kind) {
        if let Some(expr) = self.parse_infix_expression(left.clone())? {
            left = expr;
        } else {
            return Ok(left);
        }
    }

    Ok(left)
}
```

### AST Structure

The AST separates statements from expressions:

```rust
pub struct Program {
    pub statements: Vec<Statement>,
    pub source: Location,
}

pub struct Statement {
    pub kind: StatementKind,
    pub source: Location,
    pub preceded_by_blank_line: bool,
    pub trailing_comment: Option<Box<str>>,
}

pub enum StatementKind {
    Return(Box<Expression>),
    Break(Box<Expression>),
    Comment(String),
    Section { name: String, body: Box<Section>, attributes: Vec<Attribute> },
    Expression(Box<Expression>),
    Block(Vec<Statement>),
}

pub struct Expression {
    pub kind: ExpressionKind,
    pub source: Location,
}

pub enum ExpressionKind {
    Identifier(String),
    RestIdentifier(String),  // Variadic parameters (...args)
    Integer(String),
    Decimal(String),
    String(String),
    Boolean(bool),
    Nil,
    Placeholder,

    // Collections
    List(Vec<Expression>),
    Set(Vec<Expression>),
    Dictionary(Vec<(Expression, Expression)>),
    Spread(Box<Expression>),  // Spread operator in collections

    // Operators
    Prefix { operator: Prefix, right: Box<Expression> },
    Infix { operator: Infix, left: Box<Expression>, right: Box<Expression> },

    // Control flow
    If { condition: Box<Expression>, consequence: Box<Statement>, alternative: Option<Box<Statement>> },
    Match { subject: Box<Expression>, cases: Vec<MatchCase> },

    // Functions
    Function { parameters: Vec<Expression>, body: Box<Statement> },
    Call { function: Box<Expression>, arguments: Vec<Expression> },
    FunctionThread { initial: Box<Expression>, functions: Vec<Expression> },
    FunctionComposition(Vec<Expression>),

    // Ranges
    InclusiveRange { from: Box<Expression>, to: Box<Expression> },
    ExclusiveRange { from: Box<Expression>, until: Box<Expression> },
    UnboundedRange { from: Box<Expression> },

    // Bindings
    Let { name: Box<Expression>, value: Box<Expression> },
    MutableLet { name: Box<Expression>, value: Box<Expression> },
    Assign { name: Box<Expression>, value: Box<Expression> },

    // Patterns (destructuring and matching)
    IdentifierListPattern(Vec<Expression>),
    ListMatchPattern(Vec<Expression>),
    IdentifierDictionaryPattern(Vec<Expression>),
    DictionaryMatchPattern(Vec<Expression>),
    DictionaryEntryPattern { key: Box<Expression>, value: Box<Expression> },
    // ...
}
```

## Evaluator

The evaluator (`lang/src/evaluator/`) interprets the AST by walking the tree recursively.

### Value Representation

All runtime values are represented by the `Object` enum, wrapped in `Rc<Object>` for shared ownership:

```rust
pub enum Object {
    // Primitives
    Nil,
    Integer(i64),
    Decimal(OrderedFloat<f64>),
    Boolean(bool),
    String(String),

    // Collections (persistent via im-rc crate)
    List(Vector<Rc<Object>>),
    Set(HashSet<Rc<Object>>),
    Dictionary(HashMap<Rc<Object>, Rc<Object>>),
    LazySequence(LazySequence),

    // Functions
    Function(Function),

    // Internal
    Placeholder,
    Return(Rc<Object>),
    Break(Rc<Object>),
}
```

**Key design choices:**

- **Reference counting**: `Rc<Object>` enables cheap cloning and structural sharing
- **Persistent collections**: `im-rc` provides immutable data structures with efficient updates
- **OrderedFloat**: Makes floating-point values hashable for use in Sets/Dictionaries

### Environment

Lexical scoping is implemented via a linked list of environments:

```rust
pub struct Environment {
    store: Vec<(String, Rc<Object>, bool)>,   // (name, value, is_mutable)
    sections: Vec<(String, Rc<Section>, Vec<Attribute>)>,  // Runner sections
    outer: Option<EnvironmentRef>,
}

pub type EnvironmentRef = Rc<RefCell<Environment>>;
```

**Key fields:**

- **store**: Variable bindings with mutability tracking
- **sections**: AoC runner sections (`input:`, `part_one:`, `part_two:`, `test:`) with their attributes (e.g., `@slow`)
- **outer**: Parent environment for lexical scope chain

**Variable lookup** walks the chain from inner to outer scopes:

```rust
pub fn get_variable(&self, name: &str) -> Option<Rc<Object>> {
    // Check current scope
    for (name_, value, _) in &self.store {
        if name_ == name {
            return Some(Rc::clone(value));
        }
    }

    // Check enclosing scope
    if let Some(outer) = &self.outer {
        return outer.borrow().get_variable(name);
    }

    None
}
```

### Call Stack

The evaluator maintains an explicit call stack for error traces:

```rust
pub enum Frame {
    Program { environment: EnvironmentRef },
    Block { _source: Location, environment: EnvironmentRef },
    ClosureCall { source: Location, environment: EnvironmentRef },
    BuiltinCall { source: Location },
    ExternalCall { source: Location },
}
```

### Function Representation

Functions have multiple variants to support different calling conventions:

```rust
pub enum Function {
    // User-defined closure with captured environment
    Closure {
        parameters: Vec<Expression>,
        body: Statement,
        environment: EnvironmentRef,
    },

    // Closure with result caching
    MemoizedClosure {
        parameters: Vec<Expression>,
        body: Statement,
        environment: EnvironmentRef,
        cache: Rc<RefCell<HashMap<Vec<Rc<Object>>, Rc<Object>>>>,
    },

    // Built-in function (map, filter, fold, etc.)
    Builtin {
        parameters: Vec<ExpressionKind>,
        body: BuiltinFn,
        partial: Option<Arguments>,
    },

    // Runtime-injected function (puts, read)
    External {
        parameters: Vec<ExpressionKind>,
        body: ExternalFn,
        partial: Option<Arguments>,
    },

    // Composed function (f >> g >> h)
    Composition {
        functions: Vec<Function>,
    },

    // TCO trampoline marker
    Continuation {
        arguments: Vec<Rc<Object>>,
    },
}
```

**Partial application** is automatic when fewer arguments than parameters are provided:

```rust
fn apply(&self, evaluator: &mut Evaluator, arguments: Vec<Rc<Object>>, source: Location) -> Evaluation {
    // ... assign arguments to parameters ...

    if !remaining_parameters.is_empty() {
        // Return partially applied function
        return Ok(Rc::new(Object::Function(Self::Closure {
            parameters: remaining_parameters,
            body: body.clone(),
            environment: enclosed_environment,
        })));
    }

    // Fully applied - execute body
    // ...
}
```

### Tail Call Optimization

Comet implements TCO for self-recursive calls using a continuation-based trampoline:

```rust
// In eval_statement_block - detect tail calls
if let ExpressionKind::Call { function, arguments } = &expression.kind {
    for frame in self.frames.iter().rev() {
        if let Frame::ClosureCall { source, .. } = &frame {
            if function.source == *source {
                // Tail call detected - return continuation instead of recursing
                return Ok(Rc::new(Object::Function(Function::Continuation {
                    arguments: self.eval_expressions(arguments)?,
                })));
            }
            break;
        }
    }
}

// In Function::apply - trampoline loop
loop {
    if let Object::Function(Function::Continuation { arguments }) = &*result {
        // Re-bind parameters and re-execute body (no stack growth)
        self.assign_closure_parameters(enclosed_environment, parameters, arguments)?;
        result = evaluator.eval_statement(body)?;
        continue;
    }
    break;
}
```

**Limitations:**

- Only self-recursion is optimized (not mutual recursion)
- Recursion must be in true tail position

## Lazy Sequences

Lazy sequences enable working with infinite data:

```rust
pub struct LazySequence {
    value: LazyValue,
    functions: Vec<LazyFn>,
}

// Private - internal implementation detail
enum LazyValue {
    InclusiveRange { current: i64, to: i64, step: i64 },
    ExclusiveRange { current: i64, until: i64, step: i64 },
    UnboundedRange { current: i64, step: i64 },
    Repeat { value: Rc<Object> },
    Cycle { index: usize, list: Vector<Rc<Object>> },
    Iterate { current: Rc<Object>, generator: Function },
    Combinations { indices: Vec<usize>, collection: Vector<Rc<Object>> },
}

pub enum LazyFn {
    Map(Function),
    Filter(Function),
    FilterMap(Function),
    Skip(usize),
    Zip(Vec<LazySequence>),
}
```

**Composition pattern**: Transformations stack without immediate evaluation:

```rust
// 1..100 |> filter(_ % 2 == 0) |> map(_ * 2) |> take(5)

LazySequence {
    value: InclusiveRange { current: 1, to: 100, step: 1 },
    functions: vec![
        Filter(is_even_fn),
        Map(double_fn),
    ],
}
// Only first 5 matching elements are computed when take() consumes
```

## Built-in Functions

Built-ins are defined using declarative macros for consistency:

### Macro Definition

```rust
builtin! {
    map(mapper, source) [evaluator, source_loc] match {
        (Object::Function(mapper), Object::List(list)) => {
            let mut result = Vector::new();
            for element in list {
                let mapped = mapper.apply(evaluator, vec![Rc::clone(element)], source_loc)?;
                result.push_back(mapped);
            }
            Ok(Rc::new(Object::List(result)))
        }
        (Object::Function(mapper), Object::LazySequence(sequence)) => {
            Ok(Rc::new(Object::LazySequence(
                sequence.with_fn(LazyFn::Map(mapper.clone()))
            )))
        }
    }
}
```

### Registration

```rust
builtins! {
    collection::list,
    collection::map,
    collection::filter,
    collection::fold,
    collection::reduce,
    // ... extensive builtin library
}

builtin_aliases! {
    "+" => operators::plus,
    "-" => operators::minus,
    "includes?" => collection::includes,
    "type" => miscellaneous::type_name
}
```

The macros generate:

1. Parameter specification for partial application
2. Pattern-matched body with automatic type error messages
3. Registration in the global builtin lookup table

## Formatter

The formatter (`lang/src/formatter/`) produces canonical, standardized code.

### Design Philosophy

- **Opinionated**: No configuration (like `gofmt`)
- **Idempotent**: `format(format(x)) == format(x)`
- **Semantic-preserving**: Never changes program behavior

### Three-Phase Architecture

```
Source --> Parser --> AST --> Builder --> Doc --> Printer --> Formatted
                              (build)            (print)
```

### Document IR

The intermediate representation is a Wadler-style pretty-printing algebra:

```rust
pub enum Doc {
    Nil,                              // Empty
    Text(String),                     // Literal text
    Line,                             // Soft break (space or newline)
    HardLine,                         // Always newline
    BlankLine,                        // Preserved blank line
    Concat(Vec<Doc>),                 // Sequence
    Group(Box<Doc>),                  // Try to fit on one line
    Nest(usize, Box<Doc>),            // Increase indent
    IfBreak { broken: Box<Doc>, flat: Box<Doc> },  // Mode-dependent
}
```

### Printing Algorithm

The printer decides between "flat" (single-line) and "break" (multi-line) modes:

```rust
Doc::bracketed("[", elements, "]", trailing_comma: true)

// Flat mode (if it fits):
[1, 2, 3, 4, 5]

// Break mode (if too long):
[
  1,
  2,
  3,
  4,
  5,
]
```

### Style Rules

- **Line width**: 100 characters
- **Indentation**: 2 spaces
- **Pipe chains** (`|>`): Always multiline when 2+ functions
- **Collections**: Inline if fits, otherwise one-item-per-line with trailing comma

## Multi-Runtime Architecture

The core `lang` crate is runtime-agnostic. Each runtime provides:

### External Functions

Runtimes inject I/O functions via `ExternalFnDef`:

```rust
pub type ExternalFnDef = (String, Vec<ExpressionKind>, ExternalFn);
pub type ExternalFn = Rc<dyn Fn(Arguments, Location) -> Evaluation>;

// CLI runtime example
fn definitions() -> Vec<ExternalFnDef> {
    vec![
        ("puts".to_owned(),
         vec![ExpressionKind::RestIdentifier("values".to_owned())],
         Rc::new(|args, _| { /* print to stdout */ })),

        ("read".to_owned(),
         vec![ExpressionKind::Identifier("source".to_owned())],
         Rc::new(|args, _| { /* read file/URL/aoc:// */ })),
    ]
}
```

### Runtime-Specific Features

| Runtime | External Functions | Special Features |
|---------|-------------------|------------------|
| CLI | `puts`, `read`, `env` | REPL, formatting, profiling |
| WASM | `puts`, `read` | Browser/Node.js execution |
| Lambda | `puts`, `read` | AWS Lambda handler |
| Jupyter | `puts`, `read`, display | Interactive notebooks |
| PHP Extension | `puts`, `read` | PHP interop |

## Error Handling

### Error Types

```rust
// Parser error
pub struct ParserErr {
    pub message: String,
    pub source: Location,
}

// Runtime error with stack trace
pub struct RuntimeErr {
    pub message: String,
    pub source: Location,
    pub trace: Vec<Location>,
}
```

### Stack Traces

The evaluator builds traces from the call stack:

```rust
pub fn get_trace(&self) -> Vec<Location> {
    self.frames
        .iter()
        .rev()
        .filter_map(|frame| match frame {
            Frame::ClosureCall { source, .. }
            | Frame::BuiltinCall { source }
            | Frame::ExternalCall { source } => Some(*source),
            _ => None,
        })
        .collect()
}
```

## Performance Considerations

### Memory Management

- **Rc cloning**: Cheap pointer copies, no deep clones
- **Persistent collections**: Structural sharing via `im-rc`
- **Arena allocation**: `jemalloc` for improved allocation performance (CLI)

### Execution

- **No separate IR**: Direct AST interpretation (simpler but slower than bytecode)
- **Lazy sequences**: Avoid materializing large intermediate collections
- **Memoization**: Built-in caching for expensive recursive functions

### Known Trade-offs

Tree-walking is inherently slower than bytecode VMs (like Blitzen) but offers:

- Simpler implementation and debugging
- Direct source-location preservation for errors
- Easier experimentation with language features

## Testing

### Unit Tests

Each module has inline tests via `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn test_lexer_integers() {
        let lexer = Lexer::new("42 1_000");
        let tokens: Vec<_> = lexer.collect();
        expect![[r#"[Integer(42), Integer(1_000)]"#]].assert_eq(&format!("{:?}", tokens));
    }
}
```

### Integration Tests

The CLI runtime includes integration tests that run complete programs:

```rust
#[test]
fn test_aoc_solution() {
    let source = fs::read_to_string("fixtures/2023/day01.santa").unwrap();
    let result = run_solution(&source);
    assert_eq!(result.part_one, Some("54877".to_string()));
}
```

## Further Reading

- [santa-lang specification](https://eddmann.com/santa-lang/) - Language documentation
- [santa-lang-blitzen](~/Projects/santa-lang-blitzen) - Bytecode VM implementation
- [santa-lang-prancer](~/Projects/santa-lang-prancer) - TypeScript implementation
