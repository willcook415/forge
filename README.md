# Forge

Forge is a domain-specific interpreted language for engineering calculations with built-in unit safety and dimensional analysis.

## What Forge Is

Forge is designed for formula-heavy technical scripts where physical units matter.  
It treats quantities (for example `12 kN` or `300 mm^2`) as first-class values and validates dimensional correctness before execution.

## Why It Exists

Engineering calculations often fail because unit consistency is checked manually, late, or not at all.  
Forge exists to make those mistakes visible immediately with clear diagnostics.

Dimensional analysis matters because it prevents invalid operations such as adding length to time, or converting force to pressure without area context.

## Quick Start

From the project root:

```bash
# Show CLI usage
cargo run -- help

# Show version
cargo run -- version

# Validate a script (lex + parse + semantic checks only)
cargo run -- check examples/axial_stress.forge

# Execute a script
cargo run -- run examples/axial_stress.forge

# Run test suite
cargo test
```

## Example Program

```forge
# Axial stress
force = 12 kN
area = 300 mm^2
stress = force / area
print stress as MPa
```

## Example Output

```text
40 MPa
```

## Example Error

```text
Cannot add incompatible quantities. at line 4 column 1
Left operand dimension: [L]
Right operand dimension: [T]
```

## Architecture Overview

Forge executes a script through a simple pipeline:

```text
source file
  -> lexer
  -> parser
  -> AST
  -> semantic validation
  -> interpreter
  -> output / error
```

Core modules:
- `src/lexer.rs`: tokenizes source text
- `src/parser.rs`: builds AST nodes
- `src/ast.rs`: syntax tree definitions
- `src/semantic.rs`: name and dimension checks
- `src/units.rs`: quantity, dimensions, unit registry, conversions
- `src/interpreter.rs`: runtime evaluation
- `src/error.rs`: readable diagnostics with spans

Detailed design notes: `docs/architecture.md`

## Current Feature Set (v0.1)

- Assignments and print statements
- `print expr as UNIT_EXPR` conversions
- Numeric literals (integer, decimal, scientific notation)
- Variables and parentheses
- Operators: `+ - * / ^`
- Quantity literals with units
- Composed unit expressions (`mm^2`, `kg/m^3`, `kN*m`)
- CLI commands:
  - `forge run <file>`
  - `forge check <file>`
  - `forge version`

Supported built-in units:
- Base: `m`, `mm`, `s`, `kg`
- Derived: `N`, `kN`, `Pa`, `kPa`, `MPa`

## Roadmap

- More precise source spans inside expressions
- Cleaner quantity rendering for non-converted prints
- Additional engineering examples and documentation
- Future language growth beyond v0.1 scope (while preserving unit safety as the core principle)
