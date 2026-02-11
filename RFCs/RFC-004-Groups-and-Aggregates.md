# RFC-004: Groups and Aggregates

## Identifier and title

- **Identifier**: RFC-004
- **Title**: Groups and Aggregates
- **Status**: Completed

## Summary

Introduce groups as a first-class concept: developers can define groups (e.g. "Room A", "Walls", "Floor"), add and remove measurements to groups, and request aggregates—total area, total length, point count, item count—in a single requested unit. Measurements in a group may use different scales; aggregates are computed by converting each measurement’s result to the requested unit and summing. Groups do not have their own scale; they inherit from their measurements. Aggregates recompute when measurements or their scales change (no stale totals).

## Features / requirements addressed

- **F4**: Groups and aggregates (area, length, point count, item count) (Must have)
- **F2** (remaining): Aggregate requests return values normalized to the requested unit; changing a measurement’s scale updates its contribution to group totals.

## Depends on

- RFC-001 (Typed error handling)
- RFC-002 (Scale and units foundation)
- RFC-003 (Conversion and measurement kinds)

## Enables

- RFC-005 (Node and WASI API bindings)

## Complexity

High.

---

## Acceptance criteria

- [x] Developer can create groups (e.g. with a name or id) and add/remove measurements.
- [x] Adding or removing a measurement from a group updates that group’s totals.
- [x] Group aggregates: total area, total length, point count, item count—each in a single requested unit.
- [x] Measurements in a group may use different scales; each measurement’s contribution is converted to the requested unit before aggregation.
- [x] Scale is per-measurement; groups do not have a scale; totals recompute when any member’s data or scale changes.
- [x] Empty group returns zero for all aggregates (or defined behavior).
- [x] Invalid measurement references (e.g. id not found) return a typed error.
- [x] No stale aggregates; recomputation on change is required.

---

## Technical approach

- **Group**: Identity (id), collection of measurement references (ids or handles). No scale on the group itself.
- **Aggregates**: For a group, iterate over its measurements; for each, get length/area/count (using RFC-003) in the requested output unit; sum area, sum length, sum point counts, sum item counts as applicable. Cache only if explicitly scoped (e.g. immutable snapshot); otherwise compute on demand so that changes are always reflected.
- **Recomputation**: When a measurement is added/removed or a measurement’s geometry/scale changes, the next aggregate request reflects the new state. Implementation may use lazy recomputation (compute on read) or invalidate caches on write; no stale values.
- **Multi-scale**: Each measurement already has a scale (RFC-003); conversion to requested unit is done per measurement, then summed. No global scale.

## API contracts / interfaces

- Create group (id, optional name).
- Add measurement to group; remove measurement from group.
- Get aggregates: e.g. `group.total_area(unit)`, `group.total_length(unit)`, `group.point_count()`, `group.item_count()` (or single call returning a struct). All return `Result<_, TakeoffError>`.
- List groups; get group by id; list measurements in a group (if needed for API completeness).

## Data models / schema

- Group: id, name (optional), set/list of measurement ids (or references into state).
- Aggregate result: total_area, total_length, point_count, item_count (in requested unit where applicable). Document how point count and item count are defined (e.g. polyline points, count measurement items).

## State management

- Groups and membership live in the same state/store as measurements (e.g. `state` crate). Adding/removing updates state; aggregate read uses current state.
- Ensure reference integrity: removing a measurement from global state should be reflected in groups (or document that removal is a two-step: remove from groups then delete measurement).

## Error handling

- Invalid measurement id in group: typed error (e.g. `MeasurementNotFound` or reuse from RFC-001 if applicable).
- Invalid unit: `UnknownUnit` (RFC-001).
- Empty group: return zeros, not an error (per acceptance criteria).

## Testing strategy

- Unit tests: group with one measurement; add second measurement, totals update; remove measurement, totals update; two measurements with different scales, request totals in one unit, assert correct sum.
- Edge cases: empty group returns zeros; invalid measurement id returns error.
- Performance: many measurements in one group (benchmark in RFC-006).

## Implementation considerations

- Existing `takeoff_core` group module should be aligned: groups own measurement references; aggregates computed from current measurement and scale data.
- Rules from `.cursor/rules`: clarity, no duplication, type safety.

## Constraints

- Recomputation on change is required; no optional “stale allowed” for v1.
