# Forge Demo Script (30-60 seconds)

## Goal

Show that Forge can:
- run valid engineering scripts
- convert units explicitly
- reject dimensionally invalid math with clear errors

## Presenter flow

1. Open terminal in project root.
2. Run a successful engineering example:
   - `cargo run -- run examples/axial_stress.forge`
3. Run a unit conversion example:
   - `cargo run -- run examples/unit_conversion.forge`
4. Run an invalid dimensions example:
   - `cargo run -- run examples/invalid_dimensions.forge`
5. Close with:
   - "Forge catches dimensional mistakes before they reach production calculations."

## Timing guide

- Intro: 5-10s
- Valid run: 10-15s
- Unit conversion: 10-15s
- Error case: 10-15s
- Wrap-up: 5s
