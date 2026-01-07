## santa-lang Implementation

This is **Comet**, a santa-lang reindeer implementation. santa-lang is a functional programming language designed for solving Advent of Code puzzles. Multiple implementations exist to explore different execution models.

## Project Overview

- **Comet**: Tree-walking interpreter written in Rust (edition 2024)
- Workspace with core language library (`lang/`) and multiple runtime targets (CLI, WASM, Lambda, PHP ext, Jupyter)
- Batteries-included standard library for AoC patterns

## Makefile

**Always use Makefile targets.** Never run build tools directly.

- Run `make help` to see all available targets
- `make fmt` for code formatting
- `make test` for running tests
- `make can-release` before submitting a PR (runs lint + all tests)

This ensures consistent, reproducible builds across all environments (Docker, CI, local).

## Setup

```bash
# Requires Docker (recommended)
make shell              # Enter build environment
make build              # Build the project
```

## Common Commands

```bash
make help               # Show available targets
make fmt                # Format code (rustfmt)
make lint               # Run rustfmt check + clippy
make test               # Run all tests (lang, CLI, WASM)
make test/lang          # Test core language only
make test/cli           # Test CLI only
make test/wasm          # Test WebAssembly (runs on host)
make can-release        # Run before submitting PR (lint + all tests)
make bench/build        # Build benchmark Docker image
make bench/run          # Run hyperfine benchmarks
make lambda/build       # Build Lambda runtime
make jupyter/build      # Build Jupyter kernel
```

## Code Conventions

- **Edition**: Rust 2024
- **Formatting**: `max_width = 120` (rustfmt.toml)
- **Linting**: `clippy -D warnings` (deny all warnings)
- **Testing**: `expect-test` crate for snapshot testing
- **Modules**: `lexer/` → `parser/` → `evaluator/` → `formatter/` + `runner/`
- **Dependencies**: `im-rc` for persistent data structures, `ordered-float` for f64 hashing

## Tests & CI

- **CI** (`test.yml`): Runs `make can-release` on ubuntu-24.04
- **Build** (`build.yml`): Multi-platform CLI builds, Docker, WASM npm package
- Auto-updates `draft-release` branch after tests pass on main

## PR & Workflow Rules

- **Branches**: `main` for development, `draft-release` auto-updated from CI
- **CI gates**: All tests must pass before merge
- **Release**: Push to draft-release triggers build workflow

## Security & Gotchas

- **WASM tests run on host**: Requires local wasm-pack, Node.js 22, and Rust wasm32 target
- **Docker dependency**: Most targets require Docker; use `make shell` to enter build environment
- **jemalloc**: CLI uses tikv-jemallocator; may interfere with profiling
- **External functions**: `read(path)` supports file/URL/`aoc://YEAR/DAY` (requires network for AoC)

## Related Implementations

Other santa-lang reindeer (for cross-reference and consistency checks):

| Codename | Type | Language | Local Path | Repository |
|----------|------|----------|------------|------------|
| **Comet** | Tree-walking interpreter | Rust | `~/Projects/santa-lang-comet` | `github.com/eddmann/santa-lang-comet` |
| **Blitzen** | Bytecode VM | Rust | `~/Projects/santa-lang-blitzen` | `github.com/eddmann/santa-lang-blitzen` |
| **Dasher** | LLVM native compiler | Rust | `~/Projects/santa-lang-dasher` | `github.com/eddmann/santa-lang-dasher` |
| **Donner** | JVM bytecode compiler | Kotlin | `~/Projects/santa-lang-donner` | `github.com/eddmann/santa-lang-donner` |
| **Prancer** | Tree-walking interpreter | TypeScript | `~/Projects/santa-lang-prancer` | `github.com/eddmann/santa-lang-prancer` |
| **Vixen** | Embedded bytecode VM | C | `~/Projects/santa-lang-vixen` | `github.com/eddmann/santa-lang-vixen` |

Language specification and documentation: `~/Projects/santa-lang` or `github.com/eddmann/santa-lang`
