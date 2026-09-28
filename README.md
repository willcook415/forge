# Forge

Forge is a unit-safe engineering worksheet language written in Rust. It checks physical dimensions before executing calculations, with SI-based quantities, composed units and source-located diagnostics.

## A small engineering calculation

```forge
# Axial stress
force = 12 kN
area = 300 mm^2
stress = force / area
print stress as MPa
```

Output: `40 MPa`. The [example worksheets](examples) also cover beam bending, pressure vessels, shaft power, heat energy, fluid flow and Reynolds number.

## Quickstart

Install stable Rust and Cargo (Rust 2024 edition support is required), then:

```bash
git clone https://github.com/willcook415/forge.git
cd forge
cargo run --locked -- run examples/axial_stress.forge
cargo run --locked -- explain examples/axial_stress.forge
cargo run --locked -- check examples/dimension_error.forge
```

The last command intentionally fails: adding pressure to length is dimensionally invalid. It prints a source location, operand dimensions and a suggested correction.

To install the CLI from this checkout:

```bash
cargo install --locked --path .
forge new stress-check
forge run stress-check/main.forge
```

## Architecture and decisions

```text
source -> lexer -> recursive-descent parser -> AST
       -> semantic dimension analysis -> interpreter -> output
```

- Tokens and AST nodes preserve source locations for diagnostics.
- A semantic pass resolves variables, infers dimensions and rejects incompatible arithmetic/conversions before execution.
- Quantities store an `f64` SI magnitude and integer exponents for length, mass, time and temperature. Multiplication/division compose those exponents; output conversion changes the display unit.
- Parsing, dimensional analysis and execution have separate modules. The implementation has no third-party Rust dependencies.

See [architecture](docs/architecture.md) for the pipeline and [language specification](SPEC.txt) for the original language foundation.

## Validation

```bash
cargo fmt --check
cargo test --locked
```

Unit tests cover lexing, parsing, dimension inference, conversions, diagnostics and engineering example outputs. [CLI integration tests](tests/cli.rs) exercise valid worksheets, error exit paths, source snippets and project creation, including refusing to overwrite an existing directory.

[GitHub Actions](.github/workflows/ci.yml) runs formatting and tests, then builds Windows, Linux and macOS binaries. These are CI artifacts, not signed installers or a completed tagged-release system. See [release notes](docs/release.md).

Dimensional correctness catches a class of mistakes; it does not establish that a physical model, formula or safety factor is appropriate.

## CLI reference

| Command | Purpose |
| --- | --- |
| `forge run <file>` | Validate and execute a worksheet |
| `forge check <file>` | Validate without executing |
| `forge explain <file>` | Show inferred dimensions and conversions |
| `forge new <name>` | Create a starter worksheet directory |
| `forge units` | List built-in units |
| `forge examples` | List example scripts |
| `forge version` / `forge help` | Version and command help |

## Supported Units

All conversions are SI-based internally.

| Category | Units |
| --- | --- |
| Length | `m`, `mm`, `cm`, `um`, `km`, `in`, `ft` |
| Mass | `kg`, `g`, `tonne` |
| Time | `s`, `ms`, `min`, `hr` |
| Force | `N`, `kN`, `MN`, `lbf`, `kip` |
| Pressure / Stress | `Pa`, `kPa`, `MPa`, `GPa`, `bar`, `psi` |
| Volume | `L`, `mL` |
| Energy / Work | `J`, `kJ`, `Wh`, `kWh` |
| Power | `W`, `kW` |
| Temperature | `K` |
| Angle / Rotation | `rad`, `rev`, `rpm` |

Composed unit expressions are supported, including `mm^2`, `kg/m^3`, `Pa*s`, `N*m`, `kN*m`, `J/kg/K`, and `L/s`.

Forge uses `[Theta]` for temperature in dimension output. `K` is a temperature-difference unit only; absolute Celsius and Fahrenheit scales are not supported yet because they require offset conversions. Forge treats angle as dimensionless using the common SI engineering convention; `rpm` is angular speed with the standard `2*pi/60` conversion.

## Limitations

- Forge tracks length, mass, time, and temperature base dimensions in v0.2.
- Temperature support is for differences in `K`; absolute Celsius/Fahrenheit and affine offset conversions are not implemented.
- Angle is treated as dimensionless; Forge does not distinguish radians from pure scalar values in dimensional checks.
- Current, amount of substance, and luminous intensity dimensions are not implemented.
- Unit definitions are built in; v0.2 does not support user-defined units.
- Exponents must be integer literals.
- The language intentionally has no functions, loops, imports, arrays, packages, or language server yet.
- Forge checks dimensional compatibility, but it does not verify engineering formulas or safety factors.

## Further reading

[Architecture](docs/architecture.md) · [Changelog](CHANGELOG.md) · [Roadmap](ROADMAP.md) · [Release builds](docs/release.md)
