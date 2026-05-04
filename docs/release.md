# Release Builds

Forge is currently easiest to install from source:

```bash
cargo install --path .
forge version
```

## Local Release Builds

Build an optimized binary from the repository root:

```bash
cargo build --release
```

Output paths:

- Windows: `target/release/forge.exe`
- Linux/macOS: `target/release/forge`

## CI Artifacts

The GitHub Actions workflow in `.github/workflows/ci.yml` runs formatting and tests, then builds release artifacts for:

- Windows x86_64: `forge-windows-x86_64.exe`
- Linux x86_64: `forge-linux-x86_64`
- macOS arm64: `forge-macos-arm64`

These are CI build artifacts, not yet a full signed release or installer flow.

## Future Release Pass

A future pass should add tagged GitHub Releases, checksums, release notes, and any platform-specific signing or installer packaging that Forge needs.
