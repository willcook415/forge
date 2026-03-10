# Forge Architecture (v0.1)

This document explains how Forge is structured today so new contributors can navigate the codebase quickly.

## 1) End-to-End Pipeline

Forge executes scripts in a strict, linear pipeline:

```text
.forge file
  -> Lexer (src/lexer.rs)
  -> Parser (src/parser.rs)
  -> AST (src/ast.rs)
  -> Semantic validation (src/semantic.rs)
  -> Interpreter (src/interpreter.rs)
  -> printed output or error
```

CLI entrypoint (`src/main.rs`) performs:

```text
read file -> run pipeline -> print output lines
                          -> or print ForgeError and exit non-zero
```

## 2) Lexer

**File:** `src/lexer.rs`  
**Core role:** Convert source text into tokens with source positions.

Responsibilities:
- Tokenize identifiers, numbers, operators, `=`, parentheses, and keywords (`print`, `as`)
- Skip whitespace
- Skip `# ...` comments to end of line
- Preserve `line`/`column` for each token
- Emit `Eof` token at the end

Design notes:
- Numeric lexemes are kept as token text first; parsing to `f64` happens later in the parser.
- Lexer errors are immediate and include span information when available.

## 3) Parser

**File:** `src/parser.rs`  
**Core role:** Build AST nodes from tokens using recursive descent.

Supported statements:
- Assignment: `name = expression`
- Print: `print expression`
- Print with conversion: `print expression as UNIT_EXPR`

Expression precedence (highest to lowest):
1. unary minus
2. power (`^`, right-associative)
3. multiply/divide (`*`, `/`)
4. add/subtract (`+`, `-`)

Unit expression parsing:
- Base unit: `mm`
- Power: `mm^2`
- Product/division: `kN*m`, `kg/m^3`

Parser reports syntax errors with expected/found token text and line/column.

## 4) AST

**File:** `src/ast.rs`  
**Core role:** Define language structure independent of execution.

Main nodes:
- `Program { statements }`
- `Statement`
  - `Assignment { name, value, line, column }`
  - `Print { value, as_unit, line, column }`
- `Expr`
  - `Number`, `Quantity`, `Variable`, `Group`, `Unary`, `Binary`
- `UnitExpr`
  - `Unit`, `Multiply`, `Divide`, `Power`

Design notes:
- Statement-level source spans (`line`, `column`) are stored in the AST and reused by semantic and runtime diagnostics.

## 5) Semantic Validation

**File:** `src/semantic.rs`  
**Core role:** Validate program correctness before runtime execution.

Checks performed:
- Variable must be assigned before use
- `+` and `-` operands must have compatible dimensions
- `print expr as UNIT_EXPR` must be conversion-compatible
- Exponent usage:
  - exponent dimension must be dimensionless
  - exponent must be integer-literal compatible for v0.1

How it works:
- Walk AST and infer dimensions
- Maintain symbol table: `HashMap<String, Dimension>`
- Return structured errors before interpretation if invalid

## 6) Unit Registry

**File:** `src/units.rs`  
**Core role:** Define physical dimensions, quantities, and known units.

### Quantity + Dimension model

`Quantity` stores:
- `value_si: f64` (magnitude in SI base scale)
- `dimension: Dimension`

`Dimension` stores integer exponents for base dimensions:
- `L` (Length)
- `M` (Mass)
- `T` (Time)

Example display:
- `[L]`
- `[L M T^-2]`
- `[1]` (dimensionless)

### Built-in units (v0.1)

`m`, `mm`, `s`, `kg`, `N`, `kN`, `Pa`, `kPa`, `MPa`

### Unit expression resolution

`UnitRegistry::resolve_expr(UnitExpr)` recursively resolves composed units into:
- `scale_to_si`
- `dimension`

Rules:
- Multiply: scales multiply, dimensions add exponents
- Divide: scales divide, dimensions subtract exponents
- Power: scale/exponents raised to integer power

Examples:

```text
mm^2:
  scale = 0.001^2 = 1e-6
  dim   = [L]^2 = [L^2]

kg/m^3:
  scale = 1 / 1^3 = 1
  dim   = [M] / [L^3] = [L^-3 M]

kN*m:
  scale = 1000 * 1 = 1000
  dim   = [L M T^-2] * [L] = [L^2 M T^-2]
```

## 7) Interpreter

**File:** `src/interpreter.rs`  
**Core role:** Execute validated AST and produce printable lines.

Runtime model:
- Environment: `HashMap<String, Quantity>` for variables
- Evaluate expressions into `Quantity`
- Execute statements in order

Operation behavior:
- `+`/`-`: dimension-checked via quantity helpers
- `*`/`/`: compose dimensions algebraically
- `^`: exponent must be dimensionless integer
- `print expr as UNIT_EXPR`: convert and print with requested unit expression
- `print expr`: print scalar or SI value with dimension text

Formatting:
- Display formatting rounds to 6 significant figures for output only
- Internal computation remains full `f64`

## 8) Error System

**File:** `src/error.rs`  
**Core role:** Unified diagnostics type used across all stages.

`ForgeError` contains:
- plain-English message
- optional line/column span

Display behavior:
- Single-line: `Message at line X column Y`
- Multi-line details keep span on first line, then print extra context lines

Typical detailed diagnostics include:
- unknown variable / unknown unit
- invalid syntax
- incompatible dimensions (with left/right dimensions)
- invalid conversion (expression vs target dimensions)
- invalid exponent usage

## Contributor Mental Model

When adding/changing behavior, keep stage boundaries clear:
- **Lexer/Parser** define syntax and structure.
- **Semantic** enforces static validity.
- **Interpreter** executes only valid programs.
- **Units** are the source of truth for dimensional math and conversions.
- **Errors** should stay plain-English and include spans whenever possible.
