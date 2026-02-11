# TODO

## P0 - Correctness

- [ ] Implement/verify `for ... else` and `while ... else` semantics end-to-end.
  - Lowering currently carries `else_body`, but codegen paths should be audited for actual execution behavior.
- [ ] Add focused regression tests for exception + loop interactions (`break`/`continue`/`raise` inside nested `try/finally`).

## P1 - Language Coverage

- [ ] Extend import resolution beyond direct `module.py` mapping.
  - Keep the `module.py`-based model (no `__init__.py` package support planned) and improve diagnostics/module discovery within that model.
- [ ] Improve class feature support.
  - Evaluate roadmap for inheritance and richer method features.
- [ ] Expand call syntax support.
  - Keyword arguments, defaults, and broader call forms (currently restricted in lowering).

## P1 - Collections and Builtins

- [ ] Broaden list runtime API coverage (beyond `append/pop/clear`).
- [ ] Extend tuple support where currently restricted (dynamic index typing constraints).
- [ ] Add more builtin parity and richer error messages for builtin misuse.

## P2 - Tooling and DX

- [ ] Add a dedicated language-feature matrix doc generated from tests (supported vs rejected).
- [ ] Improve compile-time diagnostics with clearer actionable hints.
- [ ] Add CI checks that run `cargo test` and surface integration mismatches early.
- [ ] Add benchmarks for algorithm suite to track perf regressions across compiler/runtime changes.
