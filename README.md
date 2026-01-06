<p align="center"><a href="https://eddmann.com/santa-lang/"><img src="./docs/logo.png" alt="santa-lang" width="400px" /></a></p>

# santa-lang Comet

Tree-walking interpreter implementation of [santa-lang](https://eddmann.com/santa-lang/), written in Rust.

## Overview

santa-lang is a functional, expression-oriented programming language designed for solving Advent of Code puzzles. This Rust implementation provides a tree-walking interpreter.

Key language features:

- First-class functions and closures with tail-call optimization
- Pipeline and composition operators for expressive data flow
- Persistent immutable data structures
- Lazy sequences and infinite ranges
- Pattern matching with guards
- [Rich built-in function library](https://eddmann.com/santa-lang/builtins/)
- AoC runner with automatic input fetching

Multiple runtime targets are available: CLI, WebAssembly, AWS Lambda, PHP extension, and Jupyter kernel.

## Architecture

```
Source Code → Lexer → Parser → Evaluator → Result
                                   ↓
                           Environment (Scopes)
```

| Component       | Description                                              |
| --------------- | -------------------------------------------------------- |
| **Lexer**       | Tokenizes source into keywords, operators, literals      |
| **Parser**      | Builds an Abstract Syntax Tree (AST)                     |
| **Evaluator**   | Tree-walking interpreter that executes the AST           |
| **Environment** | Manages variable bindings and closures across scopes     |

For detailed implementation internals, see [ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Installation

### Docker

```bash
# Implementation-specific image
docker pull ghcr.io/eddmann/santa-lang-comet:cli-latest

# Generic CLI image (also published by this project)
docker pull ghcr.io/eddmann/santa-lang-cli:latest
```

### npm (WebAssembly)

```bash
npm install @eddmann/santa-lang-wasm
```

### Release Binaries

Download pre-built binaries from [GitHub Releases](https://github.com/eddmann/santa-lang-comet/releases):

| Platform              | Artifact                                     |
| --------------------- | -------------------------------------------- |
| Linux (x86_64)        | `santa-lang-comet-cli-{version}-linux-amd64` |
| Linux (ARM64)         | `santa-lang-comet-cli-{version}-linux-arm64` |
| macOS (Intel)         | `santa-lang-comet-cli-{version}-macos-amd64` |
| macOS (Apple Silicon) | `santa-lang-comet-cli-{version}-macos-arm64` |

### Other Artifacts

| Artifact                                        | Description             |
| ----------------------------------------------- | ----------------------- |
| `santa-lang-comet-wasm-{version}.tgz`           | WebAssembly npm package |
| `santa-lang-comet-lambda-{version}.zip`         | AWS Lambda layer        |
| `santa-lang-comet-php-{version}-linux-amd64.so` | PHP extension           |
| `santa-lang-comet-jupyter-{version}-{platform}` | Jupyter kernel          |

## Usage

```bash
# Run a solution
santa-cli solution.santa

# Run tests defined in a solution
santa-cli -t solution.santa

# Evaluate inline code
santa-cli -e '1 + 2'

# Interactive REPL
santa-cli -r

# Format source code
santa-cli -f solution.santa       # format to stdout
santa-cli --fmt-write solution.santa  # format in place
santa-cli --fmt-check solution.santa  # check if formatted (for CI)
```

## Example

Here's a complete Advent of Code solution (2015 Day 1):

```santa
input: read("aoc://2015/1")

part_one: {
  input |> fold(0) |floor, direction| {
    if direction == "(" { floor + 1 } else { floor - 1 };
  }
}

part_two: {
  zip(1.., input) |> fold(0) |floor, [index, direction]| {
    let next_floor = if direction == "(" { floor + 1 } else { floor - 1 };
    if next_floor < 0 { break index } else { next_floor };
  }
}

test: {
  input: "()())"
  part_one: -1
  part_two: 5
}
```

Key language features shown:

- **`input:`** / **`part_one:`** / **`part_two:`** - AoC runner sections
- **`|>`** - Pipeline operator (thread value through functions)
- **`fold`** - Reduce with early exit support via `break`
- **`test:`** - Inline test cases with expected values

## Building

Requires Rust 1.85+ or use Docker:

```bash
# Build CLI (debug)
make build

# Build CLI (release)
make release

# Run tests
make test

# Run linting
make lint
```

## Development

Run `make help` to see all available targets:

```bash
make help          # Show all targets
make shell         # Interactive shell in Docker build environment
make can-release   # Run all CI checks (lint + test)
make lint          # Run rustfmt and clippy checks
make test          # Run all tests (lang, CLI, WASM)
make fmt           # Format code
```

### Runtime Targets

```bash
# Lambda runtime
make lambda/build
make lambda/serve   # Serve locally

# PHP extension
make php-ext/build
make php-ext/test

# Jupyter kernel
make jupyter/build
make jupyter/run    # Start notebook server

# Benchmarking
make bench/build    # Build benchmark Docker image
make bench/run      # Run hyperfine benchmarks
make bench/compare V1=main V2=HEAD  # Compare versions
```

## Project Structure

```
├── lang/                  # Core language library
│   └── src/
│       ├── lexer/         # Tokenization
│       ├── parser/        # AST construction
│       ├── evaluator/     # Tree-walking interpreter
│       ├── formatter/     # Code formatting
│       └── runner/        # AoC runner support
├── runtime/
│   ├── cli/               # Command-line interface
│   ├── wasm/              # WebAssembly target
│   ├── lambda/            # AWS Lambda runtime
│   ├── php-ext/           # PHP extension
│   └── jupyter/           # Jupyter kernel
└── benchmarks/            # Performance benchmarks
```

## Other Reindeer

The language has been implemented multiple times to explore different execution models and technologies.

| Codename | Type | Language |
|----------|------|----------|
| [Comet](https://github.com/eddmann/santa-lang-comet) | Tree-walking interpreter | Rust |
| [Blitzen](https://github.com/eddmann/santa-lang-blitzen) | Bytecode VM | Rust |
| [Dasher](https://github.com/eddmann/santa-lang-dasher) | LLVM native compiler | Rust |
| [Donner](https://github.com/eddmann/santa-lang-donner) | JVM bytecode compiler | Kotlin |
| [Vixen](https://github.com/eddmann/santa-lang-vixen) | Embedded bytecode VM | C |
| [Prancer](https://github.com/eddmann/santa-lang-prancer) | Tree-walking interpreter | TypeScript |

## License

MIT License - see [LICENSE](LICENSE) for details.
