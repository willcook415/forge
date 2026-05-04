# Forge

Forge is a unit-safe engineering worksheet language for calculations where physical dimensions matter.

It lets you write small scripts with quantities such as `12 kN`, `300 mm^2`, or `9 bar`, then validates dimensional correctness before running them.

## Install

From the repository root:

```bash
cargo install --path .
forge version
```

After installation, use Forge as a normal CLI:

```bash
forge units
forge explain examples/axial_stress.forge
forge run examples/beam_bending.forge
forge check examples/dimension_error.forge
```

Release binaries are now built as CI artifacts for Windows, Linux, and macOS. A full tagged-release flow with checksums and installers is planned. See `docs/release.md`.

## Quick Start

Inspect supported units:

```bash
forge units
```

Run an engineering worksheet:

```bash
forge run examples/pressure_vessel.forge
```

Explain inferred dimensions:

```bash
forge explain examples/axial_stress.forge
```

Catch a dimensional mistake:

```bash
forge check examples/dimension_error.forge
```

## Development Fallback

You can still run everything through Cargo without installing:

```bash
cargo run -- version
cargo run -- units
cargo run -- explain examples/axial_stress.forge
cargo run -- run examples/pressure_vessel.forge
cargo run -- check examples/dimension_error.forge
cargo test
```

## Example Worksheet

```forge
# Axial stress
force = 12 kN
area = 300 mm^2
stress = force / area
print stress as MPa
```

```bash
forge run examples/axial_stress.forge
```

```text
40 MPa
```

```bash
forge explain examples/axial_stress.forge
```

```text
Inferred dimensions:
  force   [L M T^-2]
  area    [L^2]
  stress  [L^-1 M T^-2]

Outputs:
  print stress as MPa  compatible ([L^-1 M T^-2])
```

## CLI Commands

- `forge run <file>`: validate and execute a Forge script
- `forge check <file>`: validate a script without executing it
- `forge explain <file>`: show inferred dimensions and output conversions
- `forge units`: list supported built-in units
- `forge examples`: list included example scripts and suggested demo commands
- `forge version`: print the Forge version
- `forge help`: show CLI help

## What Changed In v0.2

- Expanded engineering unit registry: `cm`, `km`, `min`, `hr`, `g`, `tonne`, `MN`, `bar`, `GPa`, `J`, `kJ`, `W`, and `kW`
- `forge units` for grouped unit discovery
- `forge explain <file>` for inferred worksheet dimensions
- More realistic engineering examples for stress, bending, pressure vessels, shaft power, and Reynolds number

## Supported Units

All conversions are SI-based internally.

| Category | Units |
| --- | --- |
| Length | `m`, `mm`, `cm`, `km` |
| Mass | `kg`, `g`, `tonne` |
| Time | `s`, `min`, `hr` |
| Force | `N`, `kN`, `MN` |
| Pressure / Stress | `Pa`, `kPa`, `MPa`, `GPa`, `bar` |
| Energy / Work | `J`, `kJ` |
| Power | `W`, `kW` |

Composed unit expressions are supported, including `mm^2`, `kg/m^3`, `Pa*s`, `N*m`, and `kN*m`.

## Examples

- `examples/axial_stress.forge`: axial stress from force and area
- `examples/beam_bending.forge`: bending stress from moment, section depth, and inertia
- `examples/pressure_vessel.forge`: thin-wall hoop stress estimate
- `examples/power_torque.forge`: torque and rotational rate converted to power
- `examples/reynolds_number.forge`: dimensionless Reynolds number
- `examples/dimension_error.forge`: intentionally invalid pressure plus length
- `examples/invalid_dimensions.forge`: intentionally invalid length plus time

## Limitations

- Forge tracks only length, mass, and time base dimensions in v0.2.
- Temperature, current, amount of substance, angle semantics, offsets, and affine units are not implemented.
- Unit definitions are built in; v0.2 does not support user-defined units.
- Exponents must be integer literals.
- The language intentionally has no functions, loops, imports, arrays, packages, or language server yet.
- Forge checks dimensional compatibility, but it does not verify engineering formulas or safety factors.

## Demo Commands

Good terminal screenshots:

```bash
forge help
forge units
forge examples
forge explain examples/axial_stress.forge
forge run examples/power_torque.forge
forge check examples/dimension_error.forge
```

## Architecture

Forge executes a script through a compact pipeline:

```text
source file
  -> lexer
  -> parser
  -> AST
  -> semantic validation / dimension analysis
  -> interpreter
  -> output / error
```

Detailed design notes: `docs/architecture.md`
