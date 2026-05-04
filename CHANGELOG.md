# Changelog

## Unreleased

### Added

- Explicit `forge` binary target metadata for local installation.
- Improved CLI help text with command descriptions and example commands.
- `forge examples` command for demo-friendly example discovery.
- Compiler-style diagnostics with source snippets and caret markers for file-backed errors.
- `forge new <project-name>` command for starter worksheet projects.
- Temperature as a base dimension, displayed as `[Theta]`, with Kelvin temperature-difference unit `K`.
- Broader practical unit coverage: `um`, `in`, `ft`, `ms`, `lbf`, `kip`, `psi`, `L`, `mL`, `Wh`, `kWh`, `rad`, `rev`, and `rpm`.
- New examples for heat energy, imperial pressure conversion, shaft power with rpm, and fluid volume flow.
- GitHub Actions workflow for formatting, tests, and release binary artifacts across Windows, Linux, and macOS.
- Release build notes in `docs/release.md`.

### Changed

- README now leads with `cargo install --path .` and installed `forge ...` commands, while keeping `cargo run -- ...` as a development fallback.
- README now documents Kelvin-only temperature differences, dimensionless angle handling, and `rpm` angular-speed conversion.

## v0.2.0

Forge v0.2 turns the prototype into a unit-safe engineering worksheet language.

### Added

- Built-in units for common engineering worksheets:
  `cm`, `km`, `min`, `hr`, `g`, `tonne`, `MN`, `bar`, `GPa`, `J`, `kJ`, `W`, and `kW`
- `forge units` command with grouped unit discovery
- `forge explain <file>` command with inferred assignment dimensions and print conversion summaries
- Engineering examples for beam bending, pressure vessels, shaft power, Reynolds number, and an intentional dimension error
- Tests covering new unit conversions, grouped unit metadata, explain output, CLI behavior, and dimensional errors

### Changed

- The semantic analyzer now exposes an analysis report while preserving existing `check` and `run` behavior
- The kinetic energy example now prints directly as `kJ`
- Crate version is now `0.2.0`

### Notes

- Forge still uses an SI-based internal unit model with length, mass, and time base dimensions.
- Larger language features such as functions, loops, imports, arrays, user-defined units, packages, and language server support remain out of scope for this release.

## v0.1.0

- Initial interpreted Forge language prototype
- Assignments, print statements, arithmetic, quantity literals, composed units, semantic validation, and `run` / `check` / `version` CLI commands
