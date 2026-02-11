# RFC-005: Node and WASI API Bindings

## Identifier and title

- **Identifier**: RFC-005
- **Title**: Node and WASI API Bindings
- **Status**: Completed

## Summary

Expose a stable, backend-only API from the Rust core via Node (NAPI) and web (WASI). The API supports scales (create, list), measurements (create by kind, attach scale, get length/area/volume in unit), and groups (create, add measurements, get aggregates). Same inputs must produce same outputs from both NAPI and WASI. No UI; consumers build their own. Backend must be usable by at least one consumer app (e.g. sample-app) for quantity/cost estimation.

## Features / requirements addressed

- **F5**: Stable API from Node (NAPI) and web (WASI) (Must have)

## Depends on

- RFC-001 (Typed error handling)
- RFC-002 (Scale and units foundation)
- RFC-003 (Conversion and measurement kinds)
- RFC-004 (Groups and aggregates)

## Enables

- None (consumer-facing surface; RFC-006/007 are quality on top of core and bindings).

## Complexity

High.

---

## Acceptance criteria

- [x] Same inputs produce same outputs from NAPI and WASI bindings (consistency).
- [x] API supports: scales (create, list); measurements (create by kind, attach scale, get length/area/volume in unit); groups (create, add measurements, get aggregates).
- [x] Typed errors from RFC-001 are exposed (serializable, usable from both bindings).
- [x] Backend is usable by at least one consumer app (e.g. sample-app) for quantity/cost estimation.
- [x] No UI dependency; compatibility with Node and browser (WASI) only.
- [x] Publishable: npm package for bindings; optional crates.io for core. Semver for bindings.

---

## Technical approach

- **Layout**: Rust core in `takeoff_core`; bindings in `packages/bindings` (or equivalent), built with napi-rs for Node and WASI target for web. Single Rust codebase, two build outputs.
- **API surface**: Mirror core operations—scale create/list; measurement create (polygon, polyline, rectangle, count), set scale, get length/area/volume in unit; group create, add/remove measurement, get aggregates. Use idiomatic JS/TS types (e.g. arrays for points, objects for options).
- **Errors**: Map `TakeoffError` to JS errors or result objects with `code` and `message` (and optional details) so both NAPI and WASI expose the same shape.
- **State**: If core uses an in-memory state (e.g. state crate), bindings expose a handle or context so that multiple scales/measurements/groups can be created and referenced. Document lifecycle (e.g. create context, create scale, create measurement, attach scale, get result).
- **Build**: NAPI build for Node; WASM build for browser (WASI). CI builds both; no regressions on either target.

## API contracts / interfaces

- **Scales**: createScale(params) → scale id or object; listScales() → array of scales.
- **Measurements**: createPolygon(points), createPolyline(points), createRectangle(...), createCount(n); setScale(measurementId, scaleId); getLength(measurementId, unit), getArea(measurementId, unit), getVolume/getCount as applicable.
- **Groups**: createGroup(id/name); addMeasurement(groupId, measurementId); removeMeasurement(groupId, measurementId); getAggregates(groupId, unit) → { totalArea, totalLength, pointCount, itemCount }.
- **Errors**: Thrown or returned with type/code (e.g. InvalidScale, UnknownUnit, EmptyGeometry) and message.

## Data models / schema

- JS/TS types for scale, measurement kinds, group, aggregate result; match core types. Document in .d.ts or README.
- No persistence in v1; state is in-memory per context/session unless otherwise scoped.

## File structure

- `packages/bindings`: Rust NAPI/WASI sources, build.rs, index.js/ts entry, type definitions.
- Entry points: Node (CJS/ESM), browser (WASI worker or direct WASM load). Document usage for both.

## Error handling

- All core errors surface through bindings; no swallowed panics. Same error shape from Node and WASI.

## Testing strategy

- Bindings tests: call each API from JS/TS; assert results and errors. Run against both NAPI and WASI builds where feasible.
- Consistency test: same inputs → same outputs from NAPI and WASI (e.g. fixture of scales + measurements + groups; compare numeric results).
- Sample-app: ensure at least one consumer app (e.g. sample-app) can perform quantity/cost-oriented flows using the bindings.

## Performance

- No unnecessary copies across FFI; keep payloads minimal. Benchmark in RFC-006 can include bindings call overhead if needed.

## Implementation considerations

- napi-rs (or equivalent) for NAPI and WASM target; ensure `takeoff_core` is dependency of bindings and has no stdlib features that block WASI.
- Rules from `.cursor/rules`: async only where needed; keep bindings API sync if core is sync.
- Compatibility: Node LTS and modern browsers with WASM support.

## Third-party dependencies

- napi-rs (or current project choice); build tooling for WASI.

## Constraints

- Semver for bindings package; document breaking change policy. No UI in this package.
