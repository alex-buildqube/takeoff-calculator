# Takeoff Calculator Backend – RFC Master Index

Implementation proceeds **strictly in numerical order**. Each RFC is fully implementable only after all previous RFCs are complete. Do not implement RFC-N+1 until RFC-N is done.

---

## Sequential Implementation Order

| Order | RFC | Title | Complexity |
|-------|-----|-------|------------|
| 1 | RFC-001 | Typed error handling for invalid inputs | Medium |
| 2 | RFC-002 | Scale and units foundation | Medium |
| 3 | RFC-003 | Conversion and measurement kinds | High |
| 4 | RFC-004 | Groups and aggregates | High |
| 5 | RFC-005 | Node and WASI API bindings | High |
| 6 | RFC-006 | Performance benchmark suite | Medium |
| 7 | RFC-007 | Accuracy validation (golden / baseline tests) | Medium |
| 8 | RFC-008 | Additional units (optional / could-have) | Low |
| 9 | RFC-009 | Centroid reposition (measurement move by centroid) | Low–Medium |

---

## Dependency Graph

```
RFC-001 (Errors)
    │
    ├──► RFC-002 (Scale + Units)
    │         │
    │         ├──► RFC-003 (Conversion + Measurement kinds)
    │         │         │
    │         │         ├──► RFC-004 (Groups & aggregates)
    │         │         │         │
    │         │         └──► RFC-009 (Centroid reposition)
    │         │                   │
    │         └───────────────────┼──► RFC-005 (Node + WASI bindings)
    │                             │
    ├──► RFC-006 (Benchmarks) ◄───┘
    │
    └──► RFC-007 (Accuracy / golden tests)

RFC-008 (Additional units) — depends on RFC-002; optional for v1.
```

### Dependency table

| RFC | Depends on | Enables |
|-----|------------|---------|
| RFC-001 | — | RFC-002, RFC-003, RFC-004, RFC-005 |
| RFC-002 | RFC-001 | RFC-003, RFC-004, RFC-005, RFC-008 |
| RFC-003 | RFC-001, RFC-002 | RFC-004, RFC-005 |
| RFC-004 | RFC-001, RFC-002, RFC-003 | RFC-005 |
| RFC-005 | RFC-001, RFC-002, RFC-003, RFC-004 | — (consumer-facing) |
| RFC-006 | RFC-001–004 (core) | — |
| RFC-007 | RFC-001–004 (core) | — |
| RFC-008 | RFC-002 | — (optional) |
| RFC-009 | RFC-001, RFC-002, RFC-003 | — (extends bindings surface when implemented) |

---

## Implementation Roadmap

### Phase 1: Foundation
- **RFC-001**: Typed error handling — all invalid-input paths return clear errors; no silent failures.

### Phase 2: Core domain
- **RFC-002**: Scale (ratio + unit) and units (imperial/metric); multiple scales in context.
- **RFC-003**: Pixel → real-world conversion; polygon (Shoelace), polyline, rectangle, count; per-measurement scale.
- **RFC-004**: Groups; add/remove measurements; aggregates (area, length, point count, item count) in requested unit.

### Phase 3: API and quality
- **RFC-005**: Stable API from Node (NAPI) and web (WASI); scales, measurements, groups surface.
- **RFC-006**: Performance benchmark suite (CI).
- **RFC-007**: Accuracy validation with golden/baseline tests.

### Phase 4: Optional
- **RFC-008**: Additional units or unit systems (could-have; defer until demand).

### Phase 5: UX support
- **RFC-009**: Centroid reposition — given measurement + new centroid, return updated measurement (drag-to-move in consumer apps).

---

## File index

| RFC | Document | Implementation prompt |
|-----|----------|------------------------|
| RFC-001 | `RFC-001-Typed-Error-Handling.md` | `implementation-prompt-RFC-001.md` |
| RFC-002 | `RFC-002-Scale-and-Units-Foundation.md` | `implementation-prompt-RFC-002.md` |
| RFC-003 | `RFC-003-Conversion-and-Measurement-Kinds.md` | `implementation-prompt-RFC-003.md` |
| RFC-004 | `RFC-004-Groups-and-Aggregates.md` | `implementation-prompt-RFC-004.md` |
| RFC-005 | `RFC-005-Node-and-WASI-Bindings.md` | `implementation-prompt-RFC-005.md` |
| RFC-006 | `RFC-006-Performance-Benchmark-Suite.md` | `implementation-prompt-RFC-006.md` |
| RFC-007 | `RFC-007-Accuracy-Validation.md` | `implementation-prompt-RFC-007.md` |
| RFC-008 | `RFC-008-Additional-Units.md` | `implementation-prompt-RFC-008.md` |
| RFC-009 | `RFC-009-Centroid-Reposition.md` | `implementation-prompt-RFC-009.md` |
