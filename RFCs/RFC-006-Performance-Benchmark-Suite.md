# RFC-006: Performance Benchmark Suite

## Identifier and title

- **Identifier**: RFC-006
- **Title**: Performance Benchmark Suite
- **Status**: Completed

## Summary

Add a performance benchmark suite that runs **only in the bindings package** so that calculations run at native speed and regressions are guarded. The suite is executed via **`pnpm bench`** (from the repository root or from `packages/bindings`). Benchmarks cover representative workloads (e.g. large polygon set, many groups) and may run in CI. No significant regressions without explicit approval (e.g. tracked in CI or documented process).

## Features / requirements addressed

- **F11**: Performance benchmark suite (Should have)

## Depends on

- RFC-001 through RFC-004 (core); RFC-005 (bindings) for the bindings benchmark suite.

## Enables

- None (quality guard).

## Complexity

Medium.

---

## Acceptance criteria

- [x] Benchmark suite runs in the bindings package only; entry point is **`pnpm bench`** (from root or from `packages/bindings`).
- [x] Benchmark suite runs in CI (when CI benchmark step is added); process documented in `packages/bindings/benchmark/README.md`.
- [x] Benchmarks cover representative workloads: e.g. large polygon set, many groups.
- [x] No significant regressions without explicit approval (e.g. CI fails or flags regression; process documented).
- [x] Document how to run benchmarks locally (`pnpm bench`) and how to interpret results.
- [x] Backend-only performance (no UI); bindings package only (no separate core Rust benchmark harness in scope for this RFC).

---

## Technical approach

- **Harness**: Use a JS-callable benchmark harness in **`packages/bindings`** only (e.g. **tinybench**), driven by the existing script `benchmark/bench.mts` and run via **`pnpm bench`** (`tsx --tsconfig tsconfig.bench.json benchmark/bench.mts`). No separate Rust `criterion` harness in `takeoff_core` is in scope for this RFC.
- **Workloads**: (1) Large polygon set: e.g. hundreds or thousands of polygons; create scales and measurements, compute area/length in a loop. (2) Many groups: e.g. many groups each with multiple measurements; request aggregates for all groups. (3) Optional: bindings round-trip (create scale/measurement/group from JS, get results) to measure FFI overhead.
- **CI**: When added, run the benchmark suite in CI via `pnpm bench` (e.g. from root or `pnpm --filter ./packages/bindings bench`); either compare to baseline (e.g. previous run or checked-in baseline) or fail if execution time exceeds a threshold. Document policy: e.g. “no more than X% regression” or “benchmark run required for PR merge.”
- **Environment**: Document baseline environment (OS, CPU) if results are sensitive; optional artifact storage for historical comparison.

## API contracts / interfaces

- Benchmarks are not part of the public API; they are dev/CI tools. **Single entry point**: **`pnpm bench`** (in the bindings package; from repository root this runs via turbo). No `cargo bench` in core is in scope for this RFC.

## File structure

- **`packages/bindings/benchmark/`**: benchmark scripts (e.g. `bench.mts`), run via `pnpm bench` (script: `tsx --tsconfig tsconfig.bench.json benchmark/bench.mts`). This is the only location for the performance benchmark suite in scope for this RFC.
- **CI workflow** (when added): run `pnpm bench` (e.g. from root or with `--filter ./packages/bindings`); compare or record results.
- No `crates/takeoff_core/benches/` in scope for this RFC.

## Testing strategy

- Benchmarks themselves are the test of performance; no separate “correctness” here (covered in RFC-003, RFC-007). Optionally sanity-check that benchmark code produces correct results (e.g. assert on one known value).

## Performance

- Benchmarks should complete in a reasonable time for CI (e.g. under a few minutes). Use smaller datasets if needed for CI and document “full” dataset for local runs.

## Implementation considerations

- Avoid benchmark-only code paths that differ from production; use same APIs as production.
- Skills from `/rust-skills`: performance optimization notes; minimize async overhead if benchmarks touch async code (likely not in core).

## Constraints

- CI must be able to run the suite; no flaky or environment-dependent failures without documentation.
