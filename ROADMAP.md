# Forge Roadmap

Forge is a small, unit-safe engineering worksheet language and CLI written in Rust. It is intended for engineering learning, repeatable calculations and dimensional sanity checks, not professional design sign-off.

## Implemented (v0.1 and v0.2)

- Lexer, recursive-descent parser, AST, dimensional semantic analysis and interpreter.
- SI-based quantities, composed unit expressions, arithmetic and explicit conversions.
- Four tracked base dimensions (length, mass, time and temperature difference) and a broader built-in unit registry.
- Source-located diagnostics, `run`, `check`, `explain`, `units`, `examples`, `new`, `version` and `help` commands.
- Engineering worksheets, Rust unit/integration tests, and cross-platform CI release build artifacts.

See [CHANGELOG.md](CHANGELOG.md) and [README.md](README.md) for current behavior and limitations. The work below is prospective, not implemented.

## Possible next development milestones

These are **candidate work packages for future independent development**, not claims that the features exist or guarantees of a delivery schedule. Exact scope should be selected by the maintainer before work starts.

1. **Diagnostics and error recovery (approximately 6–10 hours).** Audit precise spans and help messages for nested expressions, malformed unit expressions and chained errors; add CLI snapshot and negative-path regression tests.
2. **Engineering calculations and validation (approximately 8–12 hours).** Expand reference examples and cross-check outputs against independently worked calculations. Add boundary and property-style tests for unit arithmetic, conversions, and rejected operations.
3. **CLI usability and deterministic output (approximately 5–8 hours).** Refine the `check` / `explain` experience, improve formatting and exit-code consistency, and test automation-friendly CLI behavior without expanding the language into a general-purpose interpreter.
4. **Release and documentation pass (approximately 5–8 hours).** Prepare tagged releases and checksums, exercise clean-install workflows on supported platforms, document known limitations and improve getting-started examples.
5. **Scoped language improvements (research and estimate before implementation).** Evaluate a small set of built-in engineering constants and clearer numeric output controls, with explicit unit semantics and regression tests.

## Explicit boundaries

- Dimensional correctness does not validate whether a formula, load case or safety factor is appropriate.
- Absolute Celsius/Fahrenheit conversions need affine semantics and are not currently supported.
- Angular quantities are dimensionless by convention; `rpm` has a standard conversion.
- Functions, loops, imports, user-defined units, large standard libraries, GUI frameworks and JIT compilation are **not** currently part of the committed scope.
- Forge does not process customer or personal data, and is not an AI-model training or evaluation project.
