# Accuracy validation and tolerance policy

This document describes how takeoff results are validated against versioned baselines and the floating-point tolerance and rounding policy used in golden tests.

## Baseline dataset

- **Location**: Baseline reference data lives in `crates/takeoff_core/test_data/baseline.json`.
- **Versioning**: The file is versioned in the repository. CI runs golden tests against this file; any change to expected values must be intentional and committed.
- **Schema**: Each baseline entry has:
  - **id**: Unique string identifier.
  - **kind**: One of `Polygon`, `Polyline`, `Rectangle`, `Count`.
  - **points**: Geometry (array of `{ "x", "y" }`; for Rectangle exactly two points; for Count exactly one).
  - **scale**: `{ "pixel_distance", "real_distance", "unit" }` (scale definition).
  - **output_unit**: Unit string for expected values (e.g. `"Feet"`, `"Meters"`).
  - **expected**: `{ "length"?, "area"?, "count"? }` — expected values in `output_unit` (only the keys applicable to the measurement kind).

See `test_data/README.md` for the full schema and examples.

## How to add or update baselines

1. Add or edit an entry in `test_data/baseline.json` following the schema.
2. Run core conversion (e.g. via a small script or the golden test harness) with the new inputs to obtain actual values.
3. Set `expected` to those values (or to the intended reference values).
4. Run `cargo test -p takeoff_core golden` to ensure the golden test passes.
5. Commit the updated `baseline.json`.

When **intended behavior changes** (e.g. formula or unit conversion fix), re-run the conversion for all affected baseline entries, update `expected` in `baseline.json`, and commit. The golden test will fail until the baseline is updated, ensuring changes are explicit.

## Tolerance and rounding policy

- **No exact float equality**: We never compare floating-point values with `==`. All comparisons use a tolerance.
- **Relative tolerance (area and length)**: For typical magnitudes we use a **relative error** of **0.01%** (1e-4). So `|actual - expected| / |expected| <= 0.0001` when `|expected|` is not too small.
- **Absolute epsilon (small values)**: When `|expected|` is very small (e.g. below a threshold such as 1e-6), relative tolerance can be too strict or undefined. We use an **absolute epsilon** (e.g. 1e-10) so that `|actual - expected| <= epsilon` is sufficient for passing.
- **Combined rule**: A baseline value passes if either:
  - `|actual - expected| <= ABSOLUTE_EPSILON`, or
  - `|actual - expected| / max(|expected|, MIN_MAGNITUDE) <= RELATIVE_TOLERANCE`.
- **Constants** (used in golden tests):
  - `RELATIVE_TOLERANCE`: 0.0001 (0.01%).
  - `ABSOLUTE_EPSILON`: 1e-10.
  - `MIN_MAGNITUDE`: threshold below which we use absolute comparison (e.g. 1e-9).

Rounding for **display** is a separate concern (e.g. in bindings or UI); the baseline stores and compares full floating-point expected values.

## Testing

- **Golden tests**: `packages/bindings/tests/golden_accuracy.rs` loads the baseline, runs core conversion for each entry, and compares results with the tolerance above. Tests fail on mismatch (no silent bypass).
- **Bindings smoke tests**: A subset of baseline cases is run through the NAPI (and WASI) bindings; results are asserted to match the same expected values (or within the same tolerance) to ensure core and bindings stay consistent.
