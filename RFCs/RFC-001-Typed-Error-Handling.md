# RFC-001: Typed Error Handling for Invalid Inputs

## Identifier and title

- **Identifier**: RFC-001
- **Title**: Typed Error Handling for Invalid Inputs
- **Status**: Completed

## Summary

Define and implement a typed error system in `takeoff_core` (and expose it through bindings) so that invalid inputs—empty geometry, zero or invalid scale, unknown unit—return clear, serializable errors instead of panics or silent wrong values. This RFC establishes the error foundation used by all subsequent RFCs.

## Features / requirements addressed

- **F10**: Typed error handling for invalid inputs (Must have)

## Depends on

- None (foundation RFC).

## Enables

- RFC-002 (Scale and units foundation)
- RFC-003 (Conversion and measurement kinds)
- RFC-004 (Groups and aggregates)
- RFC-005 (Node and WASI API bindings)

## Complexity

Medium.

---

## Acceptance criteria

- [x] Empty geometry produces a defined error type (e.g. `EmptyGeometry`).
- [x] Zero or invalid scale produces a defined error (e.g. `InvalidScale`).
- [x] Unknown or unsupported unit produces a defined error (e.g. `UnknownUnit`).
- [x] Errors are serializable and usable from both Node (NAPI) and WASI bindings.
- [x] No silent wrong values; all invalid-input paths return typed errors.
- [x] Error paths are covered by tests.

---

## Technical approach

- **Error type**: Introduce an enum (e.g. `TakeoffError`) in `takeoff_core` with variants such as `EmptyGeometry`, `InvalidScale`, `UnknownUnit`, and any other domain errors needed for v1. Use `thiserror` (or equivalent) for `Display`/`Error` and `serde` for serialization so bindings can expose them.
- **Result type**: Core APIs that can fail return `Result<T, TakeoffError>`. No panics on invalid input.
- **Validation points**: Validate at API boundaries (e.g. when creating a scale, when computing length/area, when resolving unit). Document where each variant is returned.
- **Bindings**: Map `TakeoffError` to a form that NAPI and WASI can return (e.g. JS object with `code` and `message`, or thrown error with structured properties). Ensure same behavior from both bindings.

## API contracts / interfaces

- Public functions in `takeoff_core` that accept user-controlled input return `Result<_, TakeoffError>`.
- Error enum is re-exported from the crate and (where applicable) exposed through bindings with stable names.

## Data models / schema

- `TakeoffError` enum with variants as above; optionally include context (e.g. which unit string was unknown) for diagnostics.

## Error handling

- All invalid-input cases return a variant of `TakeoffError`; no silent fallbacks that produce incorrect numbers.
- Document each variant and when it is returned; keep the set minimal for v1.

## Testing strategy

- Unit tests for each error variant: trigger the invalid input and assert the correct variant is returned.
- Tests for empty polygon/polyline, zero scale, negative scale, unknown unit string (if applicable).
- Bindings: smoke tests that invalid calls return the expected error shape from both NAPI and WASI.

## Implementation considerations

- Align with Rust rules: use `Result` and `?`; avoid `unwrap` on user input.
- Edge cases: degenerate geometry (e.g. all collinear points) may be defined as `EmptyGeometry` or a dedicated variant; document the choice.
- Rules from `.cursor/rules`: error handling and safety practices apply.

## Third-party dependencies

- `thiserror`, `serde` (if not already present) for error type.

## Constraints

- Errors must be serializable for bindings and consistent across NAPI and WASI.
