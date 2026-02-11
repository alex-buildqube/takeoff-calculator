# RFC-008: Additional Units (Optional / Could-Have)

## Identifier and title

- **Identifier**: RFC-008
- **Title**: Additional Units (Optional / Could-Have)
- **Status**: Completed

## Summary

Extend supported units or unit systems beyond the core imperial and metric set (yards, feet, inches, meters, centimeters) by extending the `Unit` type and conversion logic in core. New units can be added without breaking the existing API. Conversion math remains correct and testable. This RFC is optional for v1 and can be deferred until demand is clear.

## Features / requirements addressed

- **F8**: Additional units or unit systems (Could have)

## Depends on

- RFC-002 (Scale and units foundation)

## Enables

- None.

## Complexity

Low.

---

## Acceptance criteria

- [x] New units can be added without breaking existing API (extensible enum or registry).
- [x] Conversion math for new units is correct and testable.
- [x] Unknown or invalid unit still returns typed error (RFC-001).
- [x] Same edge-case handling as F3 (unit mismatch, unknown unit).
- [x] List of supported units is explicit in API or docs.

---

## Technical approach

- **Extension point**: Extend `Unit` enum (or unit registry) with additional length/area/volume units as needed. Use the same conversion pipeline as RFC-002 (e.g. internal reference unit → target unit).
- **Conversion**: Add conversion factors and dimensions (length/area/volume) for each new unit; ensure no duplicate or ambiguous symbols if exposing string-based unit selection.
- **API**: Existing “get length/area in unit” APIs accept the new units; no new API surface required if unit set is open-ended by design.
- **Tests**: Unit tests for new conversions; include in baseline/golden if applicable (RFC-007).

## API contracts / interfaces

- No breaking changes to existing call sites; new units are additive. Document new units in API docs and error messages (e.g. “supported units: ...”).

## Data models / schema

- `Unit` or equivalent: add variants or registry entries. Serialization (e.g. for bindings) remains stable for existing units.

## Implementation considerations

- Localization/formatting of unit names is out of scope for backend (per features.md). This RFC is about quantity conversion only.
- Defer until product demands additional units; implement when needed.

## Constraints

- Backward compatibility: existing units and behavior unchanged.
