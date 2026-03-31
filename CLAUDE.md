# CLAUDE.md

## Build & Run

```sh
cargo build              # dev build
cargo build --release    # release build
cargo install --path .   # install to ~/.cargo/bin
```

## Test

```sh
cargo test               # run all unit tests (no device required)
```

Tests cover pure functions only (parsing, escaping, deserialization).
No tests require a connected Android device or ADB server.

## Lint

```sh
cargo clippy -- -D warnings   # no warnings allowed
cargo fmt --check              # must pass before commit
```

## Prerequisites

- Rust 1.75+
- ADB server running (`adb start-server`) for runtime use
- Tesseract OCR + Leptonica dev headers for build:
  - Arch: `sudo pacman -S tesseract tesseract-data-eng`
  - Ubuntu/Debian: `sudo apt install tesseract-ocr libtesseract-dev libleptonica-dev`
  - macOS: `brew install tesseract`

## Architecture

Dual-interface tool: CLI (clap) and MCP server (rmcp, stdio transport).

Both interfaces share the same core modules:

- `adb/` - ADB protocol wrapper via `adb_client` crate (no adb binary needed)
- `screen/` - Screenshot capture, OCR (Tesseract), view hierarchy (uiautomator)
- `logcat/` - Log parsing and filtering (threadtime format)
- `input/` - Text input, tap, swipe, key events, clipboard
- `state/` - Activity state, memory info, crash reports
- `mcp/` - MCP server exposing 6 tools over stdio

## Conventions

- Error handling: `anyhow::Result` everywhere, `.context()` for actionable messages
- No `unwrap()` on user-facing paths
- All shell commands to device go through `adb::shell` / `adb::shell_str`
- Shell metacharacters must be escaped when building ADB commands (see `input::escape_for_input`)
- Single device assumed (first device returned by adb_client)
- No emdashes in any user-facing text or docs
