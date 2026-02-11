# RFC-002: Scale and Units Foundation

## Identifier and title

- **Identifier**: RFC-002
- **Title**: Scale and Units Foundation
- **Status**: Completed

## Summary

Establish the scale abstraction (ratio + unit) and the unit system (imperial and metric for length, area, and volume) in `takeoff_core`. Support multiple scales in the same context (create, list). This RFC does not implement measurement conversion; it provides the scale and unit types and conversions that RFC-003 and RFC-004 will use.

## Features / requirements addressed

- **F6**: Scale as ratio and unit; clear scale abstraction (Should have)
- **F3**: Imperial and metric units for length, area, volume (Must have)
- **F2** (partial): Multiple scales in the same context—scale creation and listing; per-measurement scale is applied when measurements are introduced in RFC-003.

## Depends on

- RFC-001 (Typed error handling)

## Enables

- RFC-003 (Conversion and measurement kinds)
- RFC-004 (Groups and aggregates)
- RFC-005 (Node and WASI API bindings)
- RFC-008 (Additional units, optional)

## Complexity

Medium.

---

## Acceptance criteria

- [x] Scale can be created with ratio (e.g. pixel_distance / real_distance or 1:120) and unit (e.g. feet).
- [x] Multiple scales can exist in the same context (create, list).
- [x] User can request length/area/volume in any supported unit; conversion from scale unit to output unit is correct.
- [x] Supported units: imperial (yards, feet, inches) and metric (meters, centimeters) for length, area, and volume where applicable.
- [x] Unknown unit returns a clear typed error (RFC-001).
- [x] Zero ratio or invalid scale parameters return typed errors (RFC-001).
- [x] Unit mismatch (e.g. requesting area in a length-only context) is handled with a clear error or defined behavior.

---

## Technical approach

- **Scale**: Represent scale as ratio (e.g. `pixel_distance / real_distance`) and a base unit. Existing `ScaleDefinition`-style types can be aligned or extended; ensure ratio and unit are explicit and validated (no zero ratio).
- **Units**: Maintain an explicit list of supported units (imperial + metric). Provide conversion between units within the same dimension (length, area, volume). Use a consistent approach (e.g. internal SI or fixed reference) so conversions are testable.
- **Multiple scales**: Context or registry allows creating and listing scales; each scale has a stable identity (e.g. id) for attachment to measurements in RFC-003.
- **Validation**: On scale creation, return `InvalidScale` for zero or negative ratio; on unit lookup, return `UnknownUnit` for unsupported units.

## API contracts / interfaces

- Scale: create (id, ratio, unit), list; ratio and unit readable.
- Units: convert value from one unit to another within same dimension; list or check supported units if needed for API clarity.

## Data models / schema

- Scale: id, ratio (or pixel_distance + real_distance), unit. No binding to a single “global” scale.
- Unit enum or type: Yards, Feet, Inches, Meters, Centimeters (and area/volume variants if modeled separately). Document which units apply to length, area, volume.

## State management

- Scales are created and stored in a context or state that RFC-003 and RFC-004 will use; exact storage shape is defined here or in RFC-003 (e.g. state crate).

## Error handling

- Invalid scale: `InvalidScale` (RFC-001).
- Unknown unit: `UnknownUnit` (RFC-001).
- No silent wrong values.

## Testing strategy

- Unit tests: scale creation with valid/invalid ratio; unit conversion accuracy (e.g. 1 ft = 12 in, 1 m = 100 cm).
- Baseline or golden values for key conversions to guard regressions.
- Multiple scales in one context: create several, list, retrieve by id.

## Implementation considerations

- Align with existing `takeoff_core` scale and unit modules; extend or refactor to meet the above. Existing `uom` usage in `unit.rs` can be retained if it supports the required units and conversions.
- Performance: conversion is pure computation; no heavy I/O.
- Rules from `.cursor/rules`: clarity, no duplication, expressive names.

## Constraints

- Same behavior from core regardless of future bindings (NAPI/WASI); no binding-specific logic in core.
