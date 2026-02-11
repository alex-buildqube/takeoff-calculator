# RFC-009: Centroid Reposition (Measurement Move by Centroid)

## Identifier and title

- **Identifier**: RFC-009
- **Title**: Centroid Reposition (Measurement Move by Centroid)
- **Status**: Draft

## Summary

Add an API that, given a measurement and a new centroid, returns the measurement with its geometry translated so the centroid lies at the given point. This enables app developers to implement "drag centroid to move measurement" in consumer apps: the backend performs the pure translation math; the app calls the API and then updates state (e.g. `set_measurement` / `upsert_measurement`). No UI or persistence is added in the backend—only the repositioning function.

**Value:** End users can drag a centroid and the measurement position updates; area, length, and count stay unchanged (translation only).

## Features / requirements addressed

- **PRD change C1** (from `prd-change-centroid-reposition.md`): Add method: given measurement + new centroid, return updated measurement (or updated points) so the shape’s centroid is at the new point; enables "drag centroid to move measurement" in consumer apps.
- **PRD goals:** Developer ergonomics, clear API (F5 surface extension); conversion/measurement (F1) — geometry remains valid for existing length/area behaviour.

## Depends on

- RFC-001 (Typed error handling) — reuse `EmptyGeometry`, etc.
- RFC-002 (Scale and units foundation) — no new scale behaviour; measurement retains scale.
- RFC-003 (Conversion and measurement kinds) — requires `Measurement`, `Point`, `get_centroid()`.

## Enables

- Consumer apps: drag-to-reposition UX without duplicating translation logic.
- Optional: future RFCs that depend on "measurement at position" (e.g. snapping, alignment) can build on this.

## Complexity

Low–Medium.

---

## Acceptance criteria

- [ ] Given a valid measurement and a new centroid point, the API returns an updated measurement whose centroid equals the given point (within floating-point tolerance).
- [ ] Area, length, and count are unchanged; only position (translation) changes.
- [ ] All measurement kinds are supported: Polygon, Polyline, Rectangle, Count. For Count, the single point becomes the new centroid.
- [ ] Empty or invalid geometry is handled with existing error types (e.g. `EmptyGeometry`).
- [ ] The API is exposed from core and from Node/WASI bindings (same shape as existing measurement APIs).
- [ ] Unit tests in core: for each kind, after repositioning, `get_centroid()` equals the requested point and area/length (where applicable) are unchanged.
- [ ] Bindings smoke test: exported API is callable and returns the expected measurement shape.

---

## Technical approach

- **Behaviour:** Pure translation. Compute current centroid via `get_centroid()`, then delta = `(new_centroid.x - current.x, new_centroid.y - current.y)`. Apply delta to every point in the measurement; return a new `Measurement` of the same variant with the same ids and scale, and the translated points.
- **Count:** The only "point" is the count location; set it to `new_centroid` (no delta needed; equivalent to moving the single point).
- **Immutability:** Do not mutate the input measurement; return a new `Measurement` (consuming or cloning as appropriate for the API style).
- **Errors:** Reuse RFC-001 errors. If `get_centroid()` fails (e.g. empty geometry), propagate that error. No new error variants required.

## API contracts / interfaces

### Core (Rust)

- **Method on `Measurement`** (recommended):  
  `fn with_centroid_at(self, new_centroid: Point) -> TakeoffResult<Measurement>`  
  Returns a new measurement of the same kind and metadata, with all points translated so the centroid is `new_centroid`. Consumes `self`.
- **Additional:** A free function in `utils` module:  
  `fn reposition_measurement_to_centroid(measurement: Measurement, new_centroid: Point) -> TakeoffResult<Measurement>`  
  if the project prefers a functional style for bindings.

### Bindings (Node / WASI)

- Expose the same capability: input = measurement + point (new centroid); output = new measurement (or equivalent points, if the team chooses points-only—recommend full measurement for `set_measurement`/`upsert_measurement`).


## Data models / schema

- No new types. Uses existing `Measurement`, `Point`, and `TakeoffResult`/`TakeoffError`.
- Return type: same `Measurement` enum; only point coordinates change.

## State management

- Stateless: input measurement + new centroid → output measurement. No server-side state; caller persists via existing update APIs.

## File structure

- **Core:** `crates/takeoff_core/src/measurement.rs` — add `with_centroid_at` (and optionally a free function in `crates/takeoff_core/src/utils.rs`).
- **Bindings:** `packages/bindings` — expose the method or function; update `index.d.ts`.

## Error handling

- Forward `get_centroid()` errors (e.g. `EmptyGeometry`). Invalid geometry for centroid computation fails before any translation.
- No new error variants.

## Testing strategy

- **Core unit tests:** For Polygon, Polyline, Rectangle, Count: build a measurement, call `with_centroid_at(new_centroid)`, assert `get_centroid() == new_centroid` (with float tolerance) and that area/length/count are unchanged.
- **Edge cases:** Empty polygon/polyline, invalid rectangle (same corners) — expect error from `get_centroid()` or `validate()`.
- **Bindings:** Smoke test that the API is exported and returns a measurement object with the expected structure (e.g. centroid matches requested point).

## Performance

- O(n) in the number of points; minimal allocations (one new measurement with translated points). No I/O. Suitable for interactive drag use.

## Security

- No new security surface; same input validation as existing measurement APIs.

## Accessibility / i18n

- N/A (backend API only).

---

## Implementation considerations

- **Return type:** Prefer returning full `Measurement` so callers can pass the result directly to `set_measurement`/`upsert_measurement` without reconstructing the measurement from points.
- **Float equality:** Use a small epsilon when asserting centroid equality in tests (e.g. `(a - b).abs() < 1e-10` or use a `approx` / `float_eq` style assertion).
- **Rules:** Follow existing `.cursor/rules` (modularity, clear names, type safety). Reuse existing patterns in `measurement.rs` and bindings.

## Third-party dependencies

- None. Uses existing `geo` (centroid already used in `get_centroid()`), `Point`, and measurement types.

## Constraints

- Backend remains UI-free; no drag handling in the backend. API only.
- Formulae and measurement kinds remain as in RFC-003; this RFC only adds translation.
