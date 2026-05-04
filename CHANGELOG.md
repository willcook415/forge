# Changelog

## Unreleased

Future changes will be tracked here.

## v0.2.0

Forge v0.2 turns the prototype into a release-ready, unit-safe engineering worksheet CLI.

### Added

- Installable standalone `forge` binary via `cargo install --path .`.
- Explicit `forge` binary target metadata.
- Improved `forge help` output with command descriptions and example commands.
- `forge examples` command for demo-friendly repository example discovery.
- Compiler-style diagnostics with source snippets, caret markers, detail lines, and help text.
- `forge new <project-name>` command for starter worksheet projects.
- `forge units` command with grouped built-in unit discovery.
- `forge explain <file>` command with inferred assignment dimensions and print conversion summaries.
- Temperature as a base dimension, displayed as `[Theta]`, with Kelvin temperature-difference unit `K`.
- Expanded engineering unit registry:
  `cm`, `um`, `km`, `in`, `ft`, `ms`, `min`, `hr`, `g`, `tonne`, `N`, `kN`, `MN`, `lbf`, `kip`, `Pa`, `kPa`, `MPa`, `GPa`, `bar`, `psi`, `L`, `mL`, `J`, `kJ`, `Wh`, `kWh`, `W`, `kW`, `rad`, `rev`, and `rpm`.
- Engineering examples for axial stress, beam bending, pressure vessels, heat energy, imperial pressure conversion, shaft power with rpm, fluid volume flow, Reynolds number, unit conversion, and intentional dimension errors.
- GitHub Actions workflow for `cargo fmt --check`, `cargo test`, and Windows/Linux/macOS release artifacts.
- Release build notes in `docs/release.md`.
- Tests covering unit conversions, dimension inference, diagnostics, project scaffolding, examples, and CLI behavior.

### Changed

- README now leads with installed `forge ...` commands while keeping `cargo run -- ...` as a development fallback.
- README documents local reinstall with `cargo install --path . --force` after source changes.
- README documents Kelvin-only temperature differences, dimensionless angle handling, and `rpm` angular-speed conversion.
- Diagnostics for incompatible addition/subtraction now place the caret on the operator when the source line is available.
- The semantic analyzer exposes an analysis report while preserving existing `check` and `run` behavior.
- The kinetic energy example now prints directly as `kJ`.
- Crate version is `0.2.0`.

### Notes

- Forge still uses an SI-based internal unit model.
- Temperature support is for differences in `K`; absolute Celsius/Fahrenheit and affine offset conversions are not implemented.
- Angle is treated as dimensionless by convention.
- Larger language features such as functions, loops, imports, arrays, user-defined units, packages, solvers, plotting, and language server support remain out of scope.

## v0.1.0

- Initial interpreted Forge language prototype.
- Assignments, print statements, arithmetic, quantity literals, composed units, semantic validation, and `run` / `check` / `version` CLI commands.
