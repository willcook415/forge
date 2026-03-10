# Forge Syntax Highlighting (VS Code)

This extension adds syntax highlighting for Forge source files (`.forge`).

## Included

- Forge language registration (`.forge` association)
- Line comments (`#`)
- Keywords (`print`, `as`)
- Numeric literals (integer, decimal, scientific notation)
- Built-in units (`m`, `mm`, `s`, `kg`, `N`, `kN`, `Pa`, `kPa`, `MPa`)
- Operators (`+`, `-`, `*`, `/`, `^`, `=`)
- Parentheses and identifiers

## Local usage

1. Open this folder in VS Code: `vscode-forge/`
2. Press `F5` to launch an Extension Development Host.
3. Open a `.forge` file in the new window.

## Install locally in VS Code

### Option 1: Install from VSIX (recommended)

1. Install packaging tool once:

```bash
npm install -g @vscode/vsce
```

2. Package the extension:

```bash
cd vscode-forge
vsce package
```

3. Install the generated `.vsix` file:

```bash
code --install-extension forge-syntax-0.1.0.vsix
```

### Option 2: Load via Extension Development Host

Use the **Local usage** steps above (`F5`) for development and quick testing without packaging.

## Packaging

If you have `vsce` installed:

```bash
cd vscode-forge
vsce package
```
