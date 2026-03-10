# Forge Roadmap

Forge is intentionally focused as an engineering calculation DSL with unit safety and dimensional analysis.  
This roadmap prioritizes high-value improvements that strengthen that focus.

## Completed in v0.1

- Core execution pipeline: lexer -> parser -> AST -> semantic validation -> interpreter
- Quantity and dimension system with SI-base representation (`L`, `M`, `T`)
- Built-in units: `m`, `mm`, `s`, `kg`, `N`, `kN`, `Pa`, `kPa`, `MPa`
- Unit expressions: `mm^2`, `kg/m^3`, `kN*m`
- Statements: assignment, `print`, `print ... as UNIT_EXPR`
- Dimensional safety checks: undefined variables, `+`/`-` compatibility, conversion compatibility, and dimensionless integer exponents
- CLI commands: `forge run <file>`, `forge check <file>`, `forge version`
- Example scripts, architecture docs, and VS Code syntax highlighting extension

## Candidate Features for v0.2

1. More precise diagnostics: track expression-level spans (not only statement-level) for sharper line/column errors.
2. Better quantity display controls: improve default rendering for non-converted prints while preserving deterministic output.
3. Additional engineering-focused built-ins: add a very small, curated set of predefined constants (for example `pi`, `g`) with clear unit semantics.
4. Stronger script checking workflow: expand `forge check` output with concise success/failure summaries suitable for CI.

## Explicitly Out of Scope

- Turning Forge into a general-purpose language
- Classes/objects, modules/import system, and large standard-library surface
- Web/server frameworks, GUI tooling, and application-runtime features
- Collections and complex data structures unrelated to core engineering calculations
- JIT/native code generation and performance-first compiler work at this stage
