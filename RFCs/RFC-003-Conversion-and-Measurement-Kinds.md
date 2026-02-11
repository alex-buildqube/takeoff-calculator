# RFC-003: Conversion and Measurement Kinds

## Identifier and title

- **Identifier**: RFC-003
- **Title**: Conversion and Measurement Kinds
- **Status**: Completed

## Summary

Implement pixel-defined geometry to real-world length, area, and volume using scale and units. Support typed measurement kinds: polygon (Shoelace area), polyline (sum of segment lengths), rectangle (width × height), and count (item count). Each measurement is associated with a scale; conversion uses that scale’s ratio and unit, and output is in a user-chosen unit. Invalid inputs (empty geometry, invalid scale) return typed errors from RFC-001.

## Features / requirements addressed

- **F1**: Pixel geometry to real-world length, area, and volume (Must have)
- **F7**: Typed measurement kinds with correct formulae (Should have)
- **F2** (partial): Per-measurement scale; measurements with different scales can coexist (scale attachment and conversion behavior).

## Depends on

- RFC-001 (Typed error handling)
- RFC-002 (Scale and units foundation)

## Enables

- RFC-004 (Groups and aggregates)
- RFC-005 (Node and WASI API bindings)

## Complexity

High.

---

## Acceptance criteria

- [x] Given valid polygon coordinates and scale, API returns area within 0.01% of baseline (exact formula: Shoelace).
- [x] Polyline length equals sum of Euclidean segment lengths, converted via scale, in chosen unit.
- [x] Rectangle area = width × height in real-world unit (from pixel dimensions and scale).
- [x] Count returns item count as provided (no geometric conversion).
- [x] Each measurement kind is explicitly typed (polygon, polyline, rectangle, count).
- [x] Empty geometry returns `EmptyGeometry` (or equivalent); invalid scale use returns `InvalidScale`.
- [x] Output can be requested in any supported unit; conversion from scale unit to output unit is correct.
- [x] Degenerate polygons (e.g. collinear points), zero-length segments, and empty shapes are handled explicitly (error or defined value, documented).

---

## Technical approach

- **Measurement kinds**: Represent as an enum or tagged type: Polygon, Polyline, Rectangle, Count. Each has the data needed for its formula (e.g. polygon: list of points; polyline: list of points; rectangle: two points or width/height in pixels; count: integer).
- **Formulae**:
  - **Polygon area**: Shoelace formula in pixel space, then multiply by (scale factor)² to get real-world area. Scale factor = real_distance / pixel_distance for a reference length (e.g. 1 pixel → real length).
  - **Polyline length**: Sum of Euclidean distances between consecutive points in pixel space; multiply by scale factor for real-world length.
  - **Rectangle**: Width and height in pixels; convert to real-world via scale; area = width × height in real-world unit.
  - **Count**: Return count as-is; no scale conversion.
- **Scale attachment**: Each measurement holds a reference to a scale (by id or value). Conversion uses that scale’s ratio and unit; output unit is a parameter to the get-length/get-area/get-volume API.
- **Units**: Use RFC-002 unit conversion to convert from scale unit to requested output unit.

## API contracts / interfaces

- Create measurement by kind (polygon, polyline, rectangle, count) with appropriate payload.
- Attach or set scale for a measurement (scale id or scale reference).
- Get length / area / volume in a given unit: `measurement.length(unit)`, `measurement.area(unit)`, etc., returning `Result<f64, TakeoffError>` (or f32 per existing codebase).
- All creation and getters validate inputs and return typed errors.

## Data models / schema

- Measurement: kind (discriminant), geometry/count data, scale reference.
- Polygon: ordered list of 2D points (pixel coords).
- Polyline: ordered list of 2D points.
- Rectangle: two corners or (width, height) in pixels.
- Count: integer count.

## Error handling

- Empty polygon/polyline (fewer than 2 points for length, fewer than 3 for polygon area): `EmptyGeometry` or documented defined value.
- Invalid or zero scale: `InvalidScale`.
- Unknown unit: `UnknownUnit`.
- All from RFC-001.

## Testing strategy

- Unit tests per kind: Shoelace vs known polygon area; polyline segment sum; rectangle area; count.
- Accuracy: compare to baseline (e.g. known polygon area within 0.01%); integrate with RFC-007 when available.
- Edge cases: degenerate polygon, zero-length polyline, empty list; expect error or documented value.
- Multiple scales: two measurements with different scales, both return correct values in a common output unit.

## Performance

- Pure computation; no I/O. Suitable for benchmarking in RFC-006.
- Prefer minimal allocations in hot paths (e.g. avoid cloning large point lists unnecessarily).

## Implementation considerations

- Existing `takeoff_core` measurement and scale modules should be aligned with this design; refactor if current types don’t support per-measurement scale or all four kinds.
- Geometry in pixel space; scale factor applied once per measurement. Clarify sign convention for Shoelace (absolute value for area).
- Rules from `.cursor/rules`: modularity, clear names, type safety.

## Constraints

- Formulae must match PRD/features: Shoelace for polygon; sum of segment lengths for polyline; width×height for rectangle; count as-is.
