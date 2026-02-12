# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Takeoff Calculator is a Rust-based backend that converts pixel-defined geometry into real-world measurements (lengths, areas, volumes). It ships as an npm package (`@build-qube/takeoff-calculator`) with native Node.js bindings via NAPI-RS and browser support via WASM.

## Monorepo Structure

- **`crates/takeoff_core/`** — Pure Rust core library. All measurement math lives here. No NAPI/JS dependencies.
- **`packages/bindings/`** — NAPI-RS bindings that wrap `takeoff_core` for Node.js and WASM (`wasm32-wasip1-threads`).
- **`packages/local-bindings/`** — Local dev wrapper for bindings.
- **`apps/sample-app/`** — React/Vite sample app for testing the library.

## Build & Development Commands

Package manager: **pnpm** (v10.24.0). Monorepo orchestration: **Turborepo**.

```bash
# Build everything (native bindings + format)
pnpm build

# Run TypeScript tests (vitest)
pnpm test

# Build and run tests (vitest)
pnpm test:local

# Run Rust tests only
pnpm test:rust

# Run both Rust and TypeScript tests
pnpm test:all

# Run a single Rust test
cargo test <test_name>

# Run a single vitest test file
pnpm --filter @build-qube/takeoff-calculator vitest run <file>

# Linting and checks
pnpm check              # lint + cargo fmt --check + clippy
pnpm lint               # biome check only
pnpm check:clippy       # cargo clippy --workspace

# Formatting (all)
pnpm format             # biome + cargo fmt + taplo (TOML)

# Benchmarks (requires build first)
pnpm bench
```

## Architecture

**Core pattern:** Rust core (`takeoff_core`) handles all computation. The bindings layer (`packages/bindings`) is a thin wrapper that converts between JS and Rust types via NAPI macros.

**Key Rust modules in `takeoff_core/src/`:**
- `state.rs` — `TakeoffStateHandler` using `DashMap` for thread-safe concurrent state (pages, groups, measurements, scales)
- `measurement.rs` — Enum-based measurement types: `Count`, `Polygon`, `Polyline`, `Rectangle`
- `scale.rs` — Scale with ratio and unit; attached per-measurement (groups inherit)
- `unit.rs` — Imperial/metric unit conversions via the `uom` crate
- `error.rs` — `TakeoffError` enum with typed variants, converts to/from NAPI errors
- `coords.rs` — Point/coordinate types
- `contour.rs` / `volume.rs` — 3D surface mesh and volume calculations

**Design decisions:**
- Scale is per-page, not global. Measurements inherit scales matching their page_id. Groups inherit scales from their measurements.
- `DashMap` for lock-free concurrent state access (no `Mutex` on the state handler).
- All invalid inputs return `Result` with `TakeoffError`; no panics on bad input.
- Geometry uses the `geo` crate (`Polygon`, `LineString`, `Rect`).

## Code Conventions

**Rust:**
- `thiserror` for error types, `anyhow` for flexible error handling
- `tokio` as async runtime; `#[tokio::test]` for async tests
- Unit tests in `#[cfg(test)]` modules within source files
- NAPI bindings use `#[napi]`, `#[napi(constructor)]`, `#[napi(string_enum)]` macros

**TypeScript/JavaScript:**
- Biome for linting and formatting: **tabs**, **double quotes**
- No unused imports (error-level, auto-fixed)
- Tests use Vitest with global API; test files in `__test__/` directories

**Pre-commit hooks (Husky + lint-staged):**
- `.rs` files: builds, formats (cargo fmt + biome), runs clippy with `--fix`
- `.js/.ts/.tsx/.json` files: `biome check --write`
- `.toml` files: `taplo format`
- Set `HUSKY=0` to skip hooks (CI does this)

## Versioning & Release

Uses `@changesets/cli`. Run `pnpm change` to create a changeset. CI auto-creates release PRs on main and publishes to npm. The bindings build for 10 platform targets (macOS, Windows, Linux variants, WASM).
