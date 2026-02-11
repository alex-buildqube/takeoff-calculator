# Product Requirements Document: Takeoff Calculator Backend

## Overview

The **Takeoff Calculator** is a Rust-based backend that converts pixel-defined geometry into real-world measurements. It gives developers a single, consistent, and accurate engine for quantity and cost estimation from drawings or design coordinates.

**Value proposition:** High speed and accuracy by running the same Rust core in both web (WASI) and Node with native performance, with first-class support for units, scales, and grouped aggregates.

---

## Goals and Objectives

| Goal                     | Description                                                                |
| ------------------------ | -------------------------------------------------------------------------- |
| **Consistency**          | One source of truth for pixel → real-world conversion across web and Node. |
| **Accuracy**             | Results that match a known baseline; validated via tests and benchmarks.   |
| **Performance**          | Native-speed computation; measurable via performance benchmarks.           |
| **Developer ergonomics** | Clear API for measurements, groups, scales, and units without UI concerns. |

---

## Scope

### In scope (initial / v1 focus)

- **Inputs:** Pixel coordinates/lengths plus scale (and support for multiple scales).
- **Outputs:** Lengths, areas, counts, and grouped aggregates in chosen units.
- **Units:** Imperial and metric (e.g. yards, feet, inches; meters, centimeters) for length, area, and volume where applicable.
- **Scale handling:** Scale ratio and unit; multiple scales per context.
- **Measurement types:** Polygons, polylines, rectangles, counts (and existing core types).
- **Groups and aggregates:** Group measurements and compute area, length, point count, and count aggregates.
- **Delivery:** Library/API only — Node (NAPI) and web (WASI) bindings; no UI.

### Out of scope (all releases)

- **Exported UI:** No shipped or supported end-user UI component; consumers build their own.

### Deferred / future

- Export formats (e.g. PDF/Excel) and persistence are not defined for v1; can be added later based on demand.

---

## User Personas / Target Audience

| Persona                             | Description                                                  | Primary need                                                    |
| ----------------------------------- | ------------------------------------------------------------ | --------------------------------------------------------------- |
| **App developer**                   | Builds takeoff, estimation, or design tools (web or Node).   | Reliable pixel → real-world math and aggregates they can embed. |
| **Integration developer**           | Wires the backend into existing estimation or CAD workflows. | Same behavior on web and Node; minimal dependency surface.      |
| **Performance-conscious developer** | Needs sub-millisecond or high-throughput calculation.        | Rust core with benchmarks to prove speed.                       |

---

## Functional Requirements

### P0 (Must-have for v1)

| ID  | Requirement                                                                                                | Notes                                                                                                                                    |
| --- | ---------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| F1  | Convert pixel geometry to real-world **length** and **area** (and volume where applicable) using a scale.  | Supports polygons, polylines, rectangles, counts.                                                                                        |
| F2  | Support **multiple scales** in the same context.                                                           | Measurements can be associated with different scales.                                                                                    |
| F3  | Support **units**: imperial (yards, feet, inches) and metric (meters, centimeters) for length/area/volume. | Output in user-chosen unit.                                                                                                              |
| F4  | **Group** measurements and compute **aggregates**: total area, total length, point count, item count.      | Groups as first-class concept with recomputation when inputs change. **Scale:** Per-measurement; groups inherit from their measurements. |
| F5  | Expose a **stable API** from **Node (NAPI)** and **web (WASI)**.                                           | No UI; backend-only surface.                                                                                                             |

### P1 (Important)

| ID  | Requirement                                                                               | Notes                                                                                                   |
| --- | ----------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| F6  | Scale has ratio and unit; measurements use scale for conversion.                          | Clear scale abstraction in core and bindings.                                                           |
| F7  | Typed measurement kinds (e.g. polygon, polyline, rectangle, count) with correct formulae. | Polygon area: Shoelace; polyline: sum of segment lengths; rectangle: width × height; count: item count. |

### P2 (Nice-to-have / future)

| ID  | Requirement                                 | Notes                                                        |
| --- | ------------------------------------------- | ------------------------------------------------------------ |
| F8  | Additional units or unit systems if needed. | Extend `Unit` and conversion in core.                        |
| F9  | Contour / 3D / volumetric extensions.       | Align with existing contour/volume modules when prioritized. |

---

## Non-Functional Requirements

| Category            | Requirement                                                                                        |
| ------------------- | -------------------------------------------------------------------------------------------------- |
| **Performance**     | Core calculations run at native speed; benchmark suite exists to guard regressions.                |
| **Accuracy**        | Results validated against known baselines (golden tests or reference data).                        |
| **Correctness**     | Type-safe Rust core; consistent behavior across NAPI and WASI.                                     |
| **Maintainability** | Clear separation: `takeoff_core` (Rust logic), `packages/bindings` (Node + WASI).                  |
| **Compatibility**   | Works in Node and in browser (WASI); no UI dependency.                                             |
| **Error handling**  | Invalid inputs (empty geometry, zero scale, unknown unit) return clear errors; no silent failures. |

---

## API and Integration

### Tech stack

| Layer         | Technology                                                      |
| ------------- | --------------------------------------------------------------- |
| Core          | Rust (`takeoff_core` crate)                                     |
| Node bindings | NAPI (e.g. `napi-rs` or equivalent)                             |
| Web bindings  | WASI / WASM (e.g. `napi-rs`)                                    |
| Publishing    | npm package for bindings; `crates.io` for Rust core (optional). |

### Key dependencies (expected)

- Rust: `serde`, unit/geometry crates as needed
- Node & WASM: napi-rs


### API surface (high-level)

- **Scales:** Create, list; ratio + unit.
- **Measurements:** Create polygon/polyline/rectangle/count; attach scale; get length/area/volume in unit; **reposition by centroid:** given a measurement and a new centroid, return the measurement (or its points) translated so its centroid is at the new point (supports e.g. drag-to-move in consumer apps).
- **Groups:** Create, add measurements; get aggregates (total area, total length, point count, item count).
- **Errors:** Typed error returns (e.g. `InvalidScale`, `UnknownUnit`, `EmptyGeometry`).

---

## User Journeys

### 1. Single measurement (length/area)

1. Developer creates a scale (e.g. 1 px = 0.1 ft).
2. Developer creates a measurement (e.g. polygon from pixel coordinates).
3. Developer attaches scale to measurement (or sets scale on context).
4. Developer reads length/area in desired unit (e.g. feet, square feet).
5. Backend returns numeric value in that unit.

**Acceptance criteria:** Given valid polygon coordinates and scale, API returns area within 0.01% of baseline; length for polyline matches sum of segment lengths in chosen unit.

### 2. Grouped aggregates for quantity estimation

1. Developer defines groups (e.g. "Room A", "Walls", "Floor").
2. Developer creates multiple measurements and assigns them to groups.
3. Developer sets scale(s) per page (measurements inherit from page).
4. Developer requests group totals: area, length, count, point count.
5. Backend recomputes aggregates when measurements or scales change.

**Acceptance criteria:** Adding/removing measurements from a group updates totals; changing a measurement’s scale updates that measurement’s contribution to group totals.

### 3. Multi-scale takeoff

1. Developer has multiple scales (e.g. plan view vs detail view).
2. Developer creates measurements in different pages and backend assigns the appropriate scale to each.
3. Developer reads all values in a single output unit.
4. Backend converts each measurement using its scale and normalizes to the chosen unit.

**Acceptance criteria:** Measurements with different scales can coexist; aggregate requests return values normalized to the requested unit.

---

## Success Metrics

| Metric          | Target / method                                                                                                                          |
| --------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| **Accuracy**    | Automated tests vs known baseline (reference lengths/areas for given pixels + scale). Versioned baseline dataset in repo or CI artifact. |
| **Performance** | Benchmark suite (e.g. large polygon set, many groups); no significant regressions.                                                       |
| **Consistency** | Same inputs produce same outputs from NAPI and WASI bindings.                                                                            |
| **Adoption**    | Backend is usable by at least one consumer app (e.g. sample-app) for quantity/cost estimation.                                           |

---

## Timeline

| Phase            | Focus                                                                             |
| ---------------- | --------------------------------------------------------------------------------- |
| **Current / v1** | Core stable: measurements, scales, units, groups, aggregates; Node + WASI; no UI. |
| **Next**         | Harden multi-scale workflows; expand benchmarks and baseline tests.               |
| **Future**       | Contour/volumetric, extra units, export or persistence if required.               |

*(Specific dates and milestones can be added when release planning is fixed.)*

---

## Testing and Release

| Aspect             | Approach                                                                                                         |
| ------------------ | ---------------------------------------------------------------------------------------------------------------- |
| **Unit tests**     | Core logic in `takeoff_core`; bindings smoke tests.                                                              |
| **Accuracy tests** | Golden/reference datasets; version in repo or CI.                                                                |
| **Benchmarks**     | Large polygon set, many groups; run in CI.                                                                       |
| **Release**        | npm package for bindings; optional `crates.io` publish for core. Semver for bindings; core follows crate semver. |

---

## Open Questions / Assumptions

| Item                  | Type         | Notes                                                                                                                              |
| --------------------- | ------------ | ---------------------------------------------------------------------------------------------------------------------------------- |
| **Cost layer**        | Assumption   | Cost estimation is applied *on top* of quantities (e.g. in app layer); backend focuses on quantities only unless otherwise scoped. |
| **Scale attachment**  | **Resolved** | Per-measurement; groups inherit from measurements.                                                                                 |
| **Baseline datasets** | Open         | Define and version reference datasets used for accuracy regression tests.                                                          |
| **Versioning**        | Assumption   | API stability and semver for the bindings package; core may follow crate semver.                                                   |

---

*PRD version: 1.1 — incorporates validation improvements: scale attachment resolution, API/tech stack, error handling, formulae clarification, and testing/release approach.*
