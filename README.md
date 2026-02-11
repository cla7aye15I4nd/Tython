# Tython

Tython is a Rust-based compiler for a statically-typed subset of Python syntax.
It parses Python source via CPython AST, lowers to a typed IR, generates LLVM IR with Inkwell, links a small C runtime, and emits a native executable.

## Current Status

- Scope: Python-like syntax with static typing requirements, not full Python compatibility.
- Codebase maturity: active development with broad feature coverage and passing current integration tests.
- Test status at the time of this update (`2026-02-11`):
  - `cargo test` passes (`3/3` integration tests passed in `tests/integration_test.rs`).
  - The large Python-level suites under `tests/basic`, `tests/imports`, and `tests/algorithm` are exercised through `tests/main.py`.

## What Works Today

Implemented and covered across `tests/basic`, `tests/imports`, and `tests/algorithm` include:

- Primitive types: `int`, `float`, `bool`
- Sequence/reference types: `str`, `bytes`, `bytearray`, `list[T]`, `tuple[...]`
- Typed functions and returns
- Classes with typed fields and methods
- Nested classes and cross-module class usage
- Control flow: `if`, `while`, `for`, `break`, `continue`
- `for` iteration over:
  - `range(...)`
  - lists
  - tuples
  - class iterators via `__iter__`/`__next__`
- List comprehensions (including nested generators and filters)
- Exception flow:
  - `try`/`except`/`finally`
  - `raise`
  - `StopIteration` handling in loops/iterator flows
- Typed operator checking and coercion for numeric ops
- Builtin support (typed subset):
  - conversions: `int`, `float`, `bool`, `str`, `bytes`, `bytearray`
  - functions: `len`, `abs`, `round`, `pow`, `min`, `max`, `repr`
  - methods: `list.append/pop/clear`, `bytearray.append/extend/clear`

## Explicit Constraints (By Design Today)

The compiler intentionally rejects unsupported or ambiguous Python patterns. Examples include:

- Missing type annotations on function parameters
- Multiple assignment targets in one statement
- Class inheritance
- Nested function definitions
- `print(...)` as an expression value
- Unsupported constants/annotations/call forms
- Invalid magic method contracts (for `__len__`, `__str__`, `__repr__`, etc.)

See `tests/invalid/` for the concrete rejection matrix currently enforced.

Import model note:

- Tython intentionally uses a direct `module.py` resolution model.
- `__init__.py` package semantics are not part of the current roadmap.

## Architecture

Compilation pipeline in `src/compiler.rs`:

1. Resolve imports (`src/resolver.rs`) with dependency ordering and cycle checks.
2. Lower Python AST to typed IR (`src/tir/lower/*`).
3. Register class layouts and generate LLVM IR (`src/codegen/context/*`).
4. Link generated bitcode with runtime (`runtime/*.c`) via `clang` and `llvm-link`.
5. Emit executable and run it from CLI entry (`src/main.rs`).

Core modules:

- `src/resolver.rs`: module path resolution and import symbol mapping
- `src/tir/`: typed intermediate representation + type rules + lowering
- `src/codegen/`: LLVM IR generation and native linking
- `runtime/`: C runtime for printing, containers, and exception machinery

## Build Requirements

You need the following on PATH:

- Rust toolchain (edition 2021)
- `python3` (used for AST parsing via `pyo3` + Python `ast` module)
- LLVM/Clang toolchain compatible with Inkwell LLVM 21 feature:
  - `clang`
  - `llvm-link`

## Build

```bash
cargo build
```

## Usage

Compile and run a Python source file:

```bash
cargo run -- path/to/main.py
```

`tython` compiles the input module graph, emits a native executable beside the input, then executes it.

## Testing

Run all tests:

```bash
cargo test
```

Important test suites:

- `tests/integration_test.rs`: compiler/runtime integration and invalid-program checks
- `tests/main.py`: high-level Python compatibility scenario
- `tests/invalid/*`: compile-time error expectations

## Repository Layout

- `src/`: compiler frontend, IR lowering, backend codegen
- `runtime/`: C runtime linked into output binaries
- `tests/`: basic features, imports, algorithm workloads, invalid programs
- `scripts/`: helper scripts/hooks
