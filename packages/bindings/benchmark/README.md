# Performance benchmark suite (RFC-006)

This directory contains the performance benchmark suite for the takeoff bindings. Benchmarks run at native speed and cover representative workloads: large polygon sets, many groups, and bindings round-trip.

## How to run benchmarks locally

From the **repository root**:

```bash
pnpm bench
```

From **this package** (`packages/bindings`):

```bash
pnpm bench
```

Both use the same entry point: `tsx --tsconfig tsconfig.bench.json benchmark/bench.mts`.

## How to interpret results

After the run, the harness prints a table with one row per benchmark:

- **Task name**: e.g. "State creation (large polygon set)", "Read area/length (large polygon set)", "Group aggregates (many groups)", "Round-trip: scale + group + measurement → group aggregate".
- **ops**: Operations per second (higher is better).
- **Average time (ms)**: Mean time per iteration (lower is better).
- **Margin / stdev**: Variability; wide margins suggest noisy or environment-dependent results.

**What to compare**: Use the same machine and load when comparing runs. A regression is a sustained increase in average time (or drop in ops) that is not explained by other processes. For CI, compare against a baseline (e.g. previous run or checked-in baseline) or a time threshold.

## Workloads

1. **Large polygon set**: State creation with 500 polygons; then reading area/length for all measurements in one group. Exercises scale application and area/length computation at scale.
2. **Many groups**: State creation with 50 groups × 20 measurements; then requesting group aggregates (area, length) for all groups. Exercises group aggregation and lookup.
3. **Round-trip**: Create one page, scale, group, and measurement from JS and read the group aggregate. Measures FFI and minimal path overhead.

Workload sizes are tuned so the full suite completes in a few minutes in CI. For deeper analysis, you can increase counts in `bench.mts` (e.g. `LARGE_POLYGON_COUNT`, `MANY_GROUPS_COUNT`, `MEASUREMENTS_PER_GROUP`) and run locally.

## CI policy

- When a CI benchmark step is added, it will run `pnpm bench` (e.g. from root or `pnpm --filter ./packages/bindings bench`).
- **No significant regressions without explicit approval**: Either the pipeline compares to a baseline (e.g. previous run or checked-in baseline) and fails if results regress beyond a threshold (e.g. >10% slower), or the process is documented (e.g. "benchmark run required for PR merge" and manual review).
- Results can be sensitive to OS and CPU; document the baseline environment if storing or comparing historical results.

## Environment

For consistent local or CI results, use a quiet machine (no heavy background load). Optional: document baseline OS and CPU when recording baselines for regression checks.
