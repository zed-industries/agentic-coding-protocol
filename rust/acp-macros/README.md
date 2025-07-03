# ACP Macros

Procedural macros for the Agentic Coding Protocol (ACP).

## Overview

This crate provides procedural macros to support the ACP implementation.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
acp-macros = { path = "rust/acp-macros" }
```

## Development

To test the macros during development:

```bash
cargo test --package acp-macros
```

To see macro expansion:

```bash
cargo expand --package agentic-coding-protocol
```

## License

MIT
