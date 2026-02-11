# Takeoff Calculator Backend – Features

## Product overview

The **Takeoff Calculator** is a Rust-based backend that converts pixel-defined geometry into real-world measurements. It provides a single source of truth for quantity (and cost-ready) estimation from drawings or design coordinates, with first-class support for units, scales, and grouped aggregates.

**Target users:** App developers and integration developers building takeoff, estimation, or design tools (web or Node). The backend runs the same Rust core in both web (WASI) and Node (NAPI) for consistency and native performance, with no shipped UI—consumers build their own.

---

## Table of contents

- [Summary](#summary)
- [By category](#by-category)
- [By priority](#by-priority)
- [Feature specifications](#feature-specifications)

---

## Summary

| Priority     | Count |
|-------------|-------|
| Must have   | 6     |
| Should have | 4     |
| Could have  | 2     |
| Won't have  | 1     |

**By category:** Conversion & measurement: 2 | Scale & units: 4 | Groups & aggregates: 1 | API & bindings: 3 | Quality & testing: 2 | Future / deferred: 1

---

## By category

| Category               | Feature IDs      |
|------------------------|------------------|
| Conversion & measurement | F1, F7        |
| Scale & units          | F2, F3, F6, F8   |
| Groups & aggregates    | F4               |
| API & bindings         | F5, F10, F13     |
| Quality & testing      | F11, F12         |
| Future / deferred      | F9               |

---

## By priority

| Priority     | Feature IDs        |
|-------------|--------------------|
| Must have   | F1, F2, F3, F4, F5, F10 |
| Should have | F6, F7, F11, F12   |
| Could have  | F8, F13           |
| Won't have  | F9                 |

---

## Feature specifications

### Conversion & measurement

#### F1: Pixel geometry to real-world length, area, and volume

- **Priority:** Must have
- **Category:** Conversion & measurement
- **Persona:** App developer, Integration developer
- **Description:** Convert pixel-defined geometry to real-world length, area, and volume using a scale. Supports polygons, polylines, rectangles, and counts. Output is in a chosen unit.
- **Acceptance criteria:**
  - Given valid polygon coordinates and scale, API returns area within 0.01% of baseline.
  - Polyline length matches sum of segment lengths in the chosen unit.
  - Rectangle and count types produce correct length/area/count in chosen unit.
- **Technical notes:** Core logic in `takeoff_core`; bindings expose results. Volume where applicable (e.g. contour/3D) may be scoped per measurement type.
- **Edge cases / special handling:** Empty geometry and invalid scale must return errors (see F10), not silent wrong values.
- **Complexity:** High
- **Integrations / risks:** Depends on scale (F2, F6) and units (F3).

#### F7: Typed measurement kinds with correct formulae

- **Priority:** Should have
- **Category:** Conversion & measurement
- **Persona:** App developer
- **Description:** Support typed measurement kinds (polygon, polyline, rectangle, count) with mathematically correct formulae: polygon area via Shoelace; polyline as sum of segment lengths; rectangle as width × height; count as item count.
- **Acceptance criteria:**
  - Polygon area matches Shoelace formula for given pixel coordinates and scale.
  - Polyline length equals sum of Euclidean segment lengths, converted via scale.
  - Rectangle area = width × height in real-world unit.
  - Count returns item count as provided.
- **Technical notes:** Clear scale abstraction in core and bindings; each kind has a single, documented formula.
- **Edge cases / special handling:** Degenerate polygons (e.g. collinear points), zero-length segments, and empty shapes handled explicitly (error or defined value).
- **Complexity:** Medium
- **Integrations / risks:** None beyond core math and scale/unit handling.

---

### Scale & units

#### F2: Multiple scales in the same context

- **Priority:** Must have
- **Category:** Scale & units
- **Persona:** App developer, Integration developer
- **Description:** Support multiple scales in the same context so measurements can be associated with different scales (e.g. plan view vs detail view) and all values can be read in a single output unit.
- **Acceptance criteria:**
  - Measurements with different scales can coexist in the same context.
  - Aggregate requests return values normalized to the requested unit.
  - Changing a measurement’s scale updates that measurement’s contribution to group totals.
- **Technical notes:** Scale is per-measurement; groups inherit from their measurements. No single “global” scale required.
- **Edge cases / special handling:** Zero or invalid scale must error (F10).
- **Complexity:** Medium
- **Integrations / risks:** Coupling with groups (F4) and units (F3).

#### F3: Imperial and metric units for length, area, volume

- **Priority:** Must have
- **Category:** Scale & units
- **Persona:** App developer
- **Description:** Support imperial (yards, feet, inches) and metric (meters, centimeters) for length, area, and volume where applicable. Output in user-chosen unit.
- **Acceptance criteria:**
  - User can request length/area/volume in any supported unit.
  - Conversion from scale unit to output unit is correct (validated via baseline or golden tests).
  - Unknown unit returns a clear error (F10).
- **Technical notes:** Extend `Unit` and conversion in core; keep list of supported units explicit in API/docs.
- **Edge cases / special handling:** Unit mismatch (e.g. area vs length) and unknown unit must return typed errors.
- **Complexity:** Medium
- **Integrations / risks:** May rely on existing unit/geometry crates; ensure consistency across NAPI and WASI.

#### F6: Scale as ratio and unit; clear scale abstraction

- **Priority:** Should have
- **Category:** Scale & units
- **Persona:** Integration developer
- **Description:** Scale is represented as ratio and unit; measurements use scale for conversion. Clear scale abstraction in core and bindings (create, list; ratio + unit).
- **Acceptance criteria:**
  - Scale can be created with ratio (e.g. 1:120) and unit (e.g. feet).
  - Measurements attach to a scale; conversion uses that scale’s ratio and unit.
  - API exposes scale creation and listing where needed.
- **Technical notes:** Align with F2 (multiple scales) and F3 (output unit); scale attachment is per-measurement (resolved in PRD).
- **Edge cases / special handling:** Zero ratio or missing unit must error.
- **Complexity:** Low
- **Integrations / risks:** None.

#### F8: Additional units or unit systems

- **Priority:** Could have
- **Category:** Scale & units
- **Persona:** App developer
- **Description:** Extend supported units or unit systems beyond core imperial and metric (e.g. additional length/area/volume units) by extending `Unit` and conversion in core.
- **Acceptance criteria:**
  - New units can be added without breaking existing API.
  - Conversion math remains correct and testable.
- **Technical notes:** Extend current unit handling; consider localization/formatting out of scope for backend.
- **Edge cases / special handling:** Same as F3 for unknown or invalid units.
- **Complexity:** Low
- **Integrations / risks:** Low; defer until demand is clear.

---

### Groups & aggregates

#### F4: Groups and aggregates (area, length, point count, item count)

- **Priority:** Must have
- **Category:** Groups & aggregates
- **Persona:** App developer
- **Description:** Group measurements and compute aggregates: total area, total length, point count, item count. Groups are first-class; aggregates recompute when inputs or scales change.
- **Acceptance criteria:**
  - Developer can define groups (e.g. "Room A", "Walls", "Floor") and add/remove measurements.
  - Adding/removing measurements from a group updates totals.
  - Group totals are returned in a single requested unit; measurements may use different scales.
  - Scale is per-measurement; groups inherit from their measurements.
- **Technical notes:** Groups own a set of measurements; no scale on group itself. Recomputation on change is required (no stale aggregates).
- **Edge cases / special handling:** Empty group returns zero aggregates; invalid measurement references must error.
- **Complexity:** High
- **Integrations / risks:** Depends on F1, F2, F3; performance with many groups should be benchmarked (F11).

---

### API & bindings

#### F5: Stable API from Node (NAPI) and web (WASI)

- **Priority:** Must have
- **Category:** API & bindings
- **Persona:** App developer, Integration developer, Performance-conscious developer
- **Description:** Expose a stable, backend-only API from Node (NAPI) and web (WASI). No UI; library/API only so consumers build their own interfaces.
- **Acceptance criteria:**
  - Same inputs produce same outputs from NAPI and WASI bindings (consistency).
  - API supports scales (create, list), measurements (create by kind, attach scale, get length/area/volume in unit), and groups (create, add measurements, get aggregates).
  - Backend is usable by at least one consumer app (e.g. sample-app) for quantity/cost estimation.
- **Technical notes:** Tech stack: Rust `takeoff_core`, NAPI (e.g. napi-rs) for Node, WASI/WASM for web. Publish npm package for bindings; optional crates.io for core. Semver for bindings.
- **Edge cases / special handling:** No UI dependency; compatibility with Node and browser only.
- **Complexity:** High
- **Integrations / risks:** NAPI and WASI build/runtime; ensure no regressions across both targets.

#### F10: Typed error handling for invalid inputs

- **Priority:** Must have
- **Category:** API & bindings
- **Persona:** Integration developer, App developer
- **Description:** Invalid inputs (empty geometry, zero scale, unknown unit) return clear, typed errors; no silent failures. Examples: `InvalidScale`, `UnknownUnit`, `EmptyGeometry`.
- **Acceptance criteria:**
  - Empty geometry returns a defined error type (e.g. `EmptyGeometry`).
  - Zero or invalid scale returns a defined error (e.g. `InvalidScale`).
  - Unknown or unsupported unit returns a defined error (e.g. `UnknownUnit`).
  - Errors are serializable and usable from both Node and WASI.
- **Technical notes:** Typed error returns in core and exposed through bindings; document all error variants.
- **Edge cases / special handling:** No silent wrong values; all error paths covered in tests.
- **Complexity:** Medium
- **Integrations / risks:** Align error shapes between Rust, NAPI, and WASI.

#### F13: Reposition measurement by centroid

- **Priority:** Could have
- **Category:** API & bindings
- **Persona:** App developer
- **Description:** Given a measurement and a new centroid, return the measurement (or its points) translated so its centroid is at the new point, enabling drag-to-move in consumer apps.
- **Acceptance criteria:** API accepts measurement + point; returns updated measurement whose centroid equals the given point; area/length/count unchanged. Empty geometry returns existing error types.
- **Technical notes:** RFC-009. Pure translation in core; expose from Node and WASI bindings.
- **Complexity:** Low

---

### Quality & testing

#### F11: Performance benchmark suite

- **Priority:** Should have
- **Category:** Quality & testing
- **Persona:** Performance-conscious developer
- **Description:** Core calculations run at native speed; a benchmark suite exists (e.g. large polygon set, many groups) and runs in CI to guard against regressions.
- **Acceptance criteria:**
  - Benchmark suite runs in CI.
  - Benchmarks cover representative workloads (e.g. large polygon set, many groups).
  - No significant regressions without explicit approval (e.g. tracked in CI).
- **Technical notes:** Use Rust benchmark harness; document how to run and interpret. No UI; backend-only performance.
- **Edge cases / special handling:** Environment variability; document baseline environment if needed.
- **Complexity:** Medium
- **Integrations / risks:** CI integration; optional artifact storage for history.

#### F12: Accuracy validation (golden / baseline tests)

- **Priority:** Should have
- **Category:** Quality & testing
- **Persona:** App developer, Integration developer
- **Description:** Results are validated against known baselines via golden or reference datasets. Versioned baseline dataset in repo or CI artifact; automated tests compare outputs to baseline.
- **Acceptance criteria:**
  - Automated tests compare core outputs to a known baseline (reference lengths/areas for given pixels + scale).
  - Baseline dataset is versioned (in repo or CI artifact).
  - Accuracy target (e.g. within 0.01% of baseline for area) is met and tested.
- **Technical notes:** Golden/reference tests in `takeoff_core`; bindings smoke tests. Define and version reference datasets (noted as open in PRD).
- **Edge cases / special handling:** Floating-point tolerance; document tolerance and rounding policy.
- **Complexity:** Medium
- **Integrations / risks:** Baseline dataset format and ownership to be defined.

---

### Future / deferred

#### F9: Contour / 3D / volumetric extensions

- **Priority:** Won't have (this release)
- **Category:** Future / deferred
- **Persona:** App developer
- **Description:** Contour, 3D, or volumetric extensions aligned with existing contour/volume modules when prioritized in a future release.
- **Acceptance criteria:**
  - Not in scope for v1; to be defined when prioritized.
- **Technical notes:** Align with existing contour/volume modules; PRD defers to future.
- **Edge cases / special handling:** N/A for v1.
- **Complexity:** High (when implemented)
- **Integrations / risks:** Depends on product decision and existing modules.
