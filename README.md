# Forge

Forge is a unit-safe engineering worksheet language for calculations where physical dimensions matter.

It lets you write small scripts with quantities such as `12 kN`, `300 mm^2`, `125 psi`, or `4180 J/kg/K`, then validates dimensional correctness before running them.

## Install

From the repository root:

```bash
cargo install --path .
forge version
```

After making local source changes, reinstall the CLI with:

```bash
cargo install --path . --force
```

After installation, use Forge as a normal CLI:

```bash
forge units
forge new stress-check
forge explain examples/axial_stress.forge
forge run examples/heat_energy.forge
forge check examples/dimension_error.forge
```

Release binaries are now built as CI artifacts for Windows, Linux, and macOS. A full tagged-release flow with checksums and installers is planned. See `docs/release.md`.

## Quick Start

Inspect supported units:

```bash
forge units
```

Run a repository example from the Forge repo root:

```bash
forge run examples/heat_energy.forge
```

Explain inferred dimensions:

```bash
forge explain examples/axial_stress.forge
```

Catch a dimensional mistake:

```bash
forge check examples/dimension_error.forge
```

## Create A Worksheet

Create a starter worksheet project:

```bash
forge new stress-check
cd stress-check
forge run main.forge
forge explain main.forge
```

Forge creates:

```text
stress-check/
  main.forge
  README.md
```

## Development Fallback

You can still run everything through Cargo without installing:

```bash
cargo run -- version
cargo run -- units
cargo run -- new stress-check
cargo run -- explain examples/axial_stress.forge
cargo run -- run examples/shaft_power_rpm.forge
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
- `forge new <project-name>`: create a starter worksheet project
- `forge units`: list supported built-in units
- `forge examples`: list included example scripts and suggested demo commands
- `forge version`: print the Forge version
- `forge help`: show CLI help

## Diagnostics

File-backed commands render compiler-style diagnostics with source snippets:

```bash
forge check examples/dimension_error.forge
```

```text
error: Cannot add incompatible quantities.
  --> examples/dimension_error.forge:4:21
   |
  4 | badtotal = pressure + length
   |                     ^
   |
   = Left operand dimension: [L^-1 M T^-2]
   = Right operand dimension: [L]
   |
   = help: addition and subtraction require matching dimensions
```

## What Changed In v0.2

- Expanded engineering unit registry for mechanical, fluids, heat-transfer, imperial, and rotational worksheets
- Temperature differences via `K`
- Angle treated as dimensionless with `rad`, `rev`, and `rpm` support
- `forge units` for grouped unit discovery
- `forge explain <file>` for inferred worksheet dimensions
- Compiler-style source diagnostics for file errors
- `forge new <project-name>` for starter worksheet projects
- More realistic engineering examples for stress, bending, pressure vessels, shaft power, and Reynolds number

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

## Examples

These scripts live in the repository `examples/` directory. The suggested `forge run examples/...` commands assume you are running from the Forge repository root.

- `examples/axial_stress.forge`: axial stress from force and area
- `examples/beam_bending.forge`: bending stress from moment, section depth, and inertia
- `examples/pressure_vessel.forge`: thin-wall hoop stress estimate
- `examples/power_torque.forge`: torque and rotational rate converted to power
- `examples/shaft_power_rpm.forge`: shaft power from torque and `rpm`
- `examples/heat_energy.forge`: heat energy from mass, specific heat, and temperature rise
- `examples/imperial_pressure.forge`: pressure conversion from `psi`
- `examples/fluid_volume_flow.forge`: volumetric flow rate using litres
- `examples/reynolds_number.forge`: dimensionless Reynolds number
- `examples/dimension_error.forge`: intentionally invalid pressure plus length
- `examples/invalid_dimensions.forge`: intentionally invalid length plus time

## Limitations

- Forge tracks length, mass, time, and temperature base dimensions in v0.2.
- Temperature support is for differences in `K`; absolute Celsius/Fahrenheit and affine offset conversions are not implemented.
- Angle is treated as dimensionless; Forge does not distinguish radians from pure scalar values in dimensional checks.
- Current, amount of substance, and luminous intensity dimensions are not implemented.
- Unit definitions are built in; v0.2 does not support user-defined units.
- Exponents must be integer literals.
- The language intentionally has no functions, loops, imports, arrays, packages, or language server yet.
- Forge checks dimensional compatibility, but it does not verify engineering formulas or safety factors.

## Demo Commands

Good terminal screenshots from the Forge repository root:

```bash
forge help
forge units
forge examples
forge new stress-check
forge explain examples/axial_stress.forge
forge run examples/heat_energy.forge
forge run examples/shaft_power_rpm.forge
forge run examples/imperial_pressure.forge
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
