# Tython

> A statically-typed, compiled Python variant with immutable references

## What is Tython?

Tython is a programming language that maintains **full compatibility with Python 3 syntax** while introducing two key constraints that enable compilation to highly optimized machine code:

- **Static Type Inference**: All variable types must be inferrable at compile time
- **Immutable References**: References cannot be modified after initialization. It is worth noting that references located at stack scope are mutable, because tython treat them as renaming of variables.

By enforcing these constraints, Tython achieves the performance of compiled languages while preserving the elegant syntax and semantics that make Python beloved by developers.

---

## Features

- **Python-Compatible Syntax**: Write code that looks and feels like Python
- **Compile-Time Type Inference**: No runtime type checking overhead
- **LLVM Backend**: Leverages LLVM for world-class optimization and code generation
- **Cross-File Optimization**: Global symbol table enables optimization across module boundaries
- **Single Binary Output**: Compiles your entire project into a standalone executable

## Architecture

Tython is implemented in **Rust** and generates a CLI tool (`tython`) that transforms `.py` files into optimized binary executables.

### Compilation Pipeline

```
Input.py → AST → Import Resolution → Type Inference → TIR → LLVM IR → Binary
```

#### Detailed Workflow

1. **Parsing**: Convert source code into an Abstract Syntax Tree (AST) using Python's built-in `ast` module

2. **Dependency Resolution**: Recursively process all import statements in depth-first order

3. **Type Inference & TIR Generation**: Analyze the AST to infer types for all variables and expressions, then transform into Tython Intermediate Representation (TIR)

4. **LLVM IR Generation**: Convert TIR into LLVM Intermediate Representation and merge into the global LLVM module

5. **Optimization & Compilation**: Apply LLVM optimizations and compile the module into a binary executable

### Global Symbol Table

Tython maintains a unified symbol table across all files, enabling:
- Cross-module type checking
- Inter-procedural optimization
- Dead code elimination across boundaries

---

## Testing

The test suite is designed as a comprehensive integration test that validates Tython's compatibility with Python.

- **Entry Point**: [`tests/main.py`](tests/main.py)
- **Test Strategy**: Each test case is executed by both `tython` and `python`, and their outputs are compared to ensure identical behavior

This approach guarantees that Tython programs produce the same results as their Python equivalents.