# RFC-007: Accuracy Validation (Golden / Baseline Tests)

## Identifier and title

- **Identifier**: RFC-007
- **Title**: Accuracy Validation (Golden / Baseline Tests)
- **Status**: Completed

## Summary

Validate takeoff results against known baselines using golden or reference datasets. Automated tests compare core outputs (length, area, volume where applicable) to a versioned baseline for given pixel geometry and scale. Accuracy target (e.g. within 0.01% of baseline for area) is met and tested. Baseline dataset is versioned in repo or as CI artifact.

## Features / requirements addressed

- **F12**: Accuracy validation (golden / baseline tests) (Should have)

## Depends on

- RFC-001 through RFC-004 (core logic to validate)

## Enables

- None (quality guard).

## Complexity

Medium.

---

## Acceptance criteria

- [x] Automated tests compare core outputs to a known baseline (reference lengths/areas for given pixels + scale).
- [x] Baseline dataset is versioned (in repo or CI artifact).
- [x] Accuracy target (e.g. within 0.01% of baseline for area) is met and tested.
- [x] Floating-point tolerance and rounding policy are documented.
- [x] Golden/reference tests live in `takeoff_core`; bindings have smoke tests that exercise the same flows.

---

## Technical approach

- **Baseline format**: Define a format for reference data: input (e.g. polygon points, scale, unit) and expected output (length, area). Store as JSON, YAML, or code (e.g. Rust test data). Version in repo or store as CI artifact with version tag.
- **Test harness**: For each baseline entry, run core conversion (RFC-003) with the given inputs and compare result to expected value using a tolerance (e.g. relative 0.01% or absolute epsilon). Fail test if outside tolerance.
- **Coverage**: Polygon (Shoelace), polyline (segment sum), rectangle, count; multiple scales and units. Include edge cases (e.g. simple square, degenerate shapes if defined).
- **Tolerance**: Document policy (e.g. relative error for area/length; absolute for small values; no exact float equality). Use a small epsilon or relative diff.
- **Bindings**: Smoke tests that run the same (or a subset of) baseline cases through NAPI/WASI and assert same results as core.

## API contracts / interfaces

- No new public API; tests use existing core API. Baseline files or modules are test-only.

## Data models / schema

- Baseline entry: inputs (geometry, scale, unit), expected outputs (length, area, count as applicable). Schema documented so new baselines can be added.

## File structure

- `crates/takeoff_core/tests/` or `crates/takeoff_core/src/` test modules: golden/baseline tests.
- `crates/takeoff_core/test_data/` or `tests/fixtures/`: baseline files (versioned).
- Document where baselines live and how to add/update them.

## Error handling

- Tests expect success for valid baseline entries; invalid entries should not be in baseline set. Document that baseline format is validated separately if needed.

## Testing strategy

- Golden tests: load baseline, run core, compare with tolerance. Fail on mismatch.
- Document how to update baselines when intended behavior changes (e.g. re-run and commit new expected values).
- Bindings smoke: run subset of baselines through bindings; assert consistency with core.

## Implementation considerations

- PRD notes “baseline datasets” as open: this RFC defines a minimal format and location; can be extended later.
- Rules from `.cursor/rules`: testing, clarity. Avoid hardcoding magic numbers; use named constants for tolerance.

## Constraints

- Baseline dataset format and ownership documented; no silent tolerance bypass (tests must fail when accuracy regresses).
