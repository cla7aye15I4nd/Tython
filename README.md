# Tython

![Rust](https://img.shields.io/badge/Rust-2021-black?logo=rust)
![LLVM](https://img.shields.io/badge/LLVM-21-blue)
![Compiler](https://img.shields.io/badge/Mode-AOT%20Native-success)
![Status](https://img.shields.io/badge/Status-Experimental-orange)

Tython compiles a statically typed subset of Python directly to native executables.

It uses CPython AST parsing, lowers to a typed IR, generates LLVM IR with Inkwell, links a custom runtime, and emits a static binary.

## Why It Is Interesting

- Python-like source with compiler-grade static checks
- Native output instead of bytecode/JIT
- Strict, explicit unsupported matrix in `tests/invalid/*`
- Real algorithm and stress suites in `tests/basic`, `tests/algorithm`, `tests/classes`, and `tests/imports`

## At A Glance

| Area | Details |
|---|---|
| Language Model | Typed Python subset (not full CPython compatibility) |
| Frontend | CPython `ast` via `pyo3` |
| IR | Typed IR (`src/tir/*`) |
| Backend | LLVM IR via Inkwell |
| Runtime | C++ runtime (`runtime/*`) |
| Binaries | `tython`, `pycmin` |

## Quick Example

```python
def fib(n: int) -> int:
    a: int = 0
    b: int = 1
    i: int = 0
    while i < n:
        tmp: int = b
        b = a + b
        a = tmp
        i = i + 1
    return a

print(fib(10))
```

Compile and run:

```bash
cargo run -- path/to/main.py
```

Tython emits an executable next to the input file, then runs it.

## Pipeline

1. Resolve imports and module graph (`src/resolver.rs`)
2. Lower Python AST into typed IR (`src/tir/lower/*`)
3. Emit LLVM IR and class layouts (`src/codegen/*`)
4. Optimize and link runtime (`src/codegen/mod.rs`, `runtime/*`)
5. Build runtime entry and execute (`src/main.rs`)

## Feature Matrix

| Feature | Status |
|---|---|
| `int`, `float`, `bool` | Implemented |
| `str`, `bytes`, `bytearray` | Implemented |
| `list`, `tuple`, `dict`, `set` | Implemented (typed subset) |
| Typed functions + defaults + kwargs (regular calls) | Implemented |
| Classes + magic methods subset | Implemented |
| `if` / `while` / `for` + loop `else` | Implemented |
| `try` / `except` / `else` / `finally` | Implemented |
| Comprehensions + iterator flows | Implemented |
| Inheritance | Not supported |
| Package `__init__.py` import model | Not supported |

## Known Limits

- Import resolution is `module.py` based, not package-style `__init__.py` (`src/resolver.rs`)
- Inheritance is rejected (`tests/invalid/class_inheritance/main.py`)
- Keyword-only, positional-only, varargs, and `**kwargs` parameters are rejected (`src/tir/lower/functions.rs`)
- Constructor/method keyword-call forms are rejected (`src/tir/lower/call/resolve.rs`)
- Indirect calls through function values are rejected (`src/tir/lower/call/resolve.rs`)
- `return` inside `try/finally` contexts is rejected (`src/tir/lower/stmt/core.rs`)

## Testing

Run:

```bash
cargo test
```

Primary coverage entry points:

- `tests/integration_test.rs`
- `tests/main.py`
- `tests/invalid/*`

Audit notes from 2026-02-16:

- `pycmin` currently has no automated test coverage (`src/bin/pycmin.rs`)
- Invalid tests assert failure only, not exact diagnostic category/message (`tests/integration_test.rs`)
- Dead branch exists in `tests/basic/test_exception_iter_stress.py` (`test_else_branch_with_stopiteration`)

## Build Requirements

Tools expected on `PATH`:

- Rust toolchain (edition 2021)
- `python3`
- `llvm-as`
- `clang++`

## Build

```bash
cargo build
```

## Repository Layout

- `src/`: compiler frontend, lowering, codegen, CLI
- `runtime/`: native runtime library
- `tests/`: compatibility suites + invalid corpus
- `stdlib/`: bundled stdlib modules used by resolver
- `scripts/`: helper utilities
