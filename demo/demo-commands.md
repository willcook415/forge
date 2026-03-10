# Forge Demo Commands

Run these from the repository root.

## 1) Successful example run

```bash
cargo run -- run examples/axial_stress.forge
```

Expected output:

```text
40 MPa
```

## 2) Unit conversion run

```bash
cargo run -- run examples/unit_conversion.forge
```

Expected output:

```text
2.5 m
```

## 3) Dimensional error example

```bash
cargo run -- run examples/invalid_dimensions.forge
```

Expected output (error):

```text
Cannot add incompatible quantities. at line 4 column 1
Left operand dimension: [L]
Right operand dimension: [T]
```
