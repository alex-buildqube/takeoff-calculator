# Update Contour Input: Unit-Based Elevation

## Problem

Contour elevation values are real-world measurements (feet, meters), not pixel values. The current `ContourLineInput` and `ContourPointOfInterestInput` treat elevation as a raw `f64` with no unit information. This means the surface mesh is built with pixel-scale elevations, producing incorrect volumetric results.

## Decision Summary

- Elevation becomes `f64` + `Unit` (explicit unit per contour line/POI)
- Contours are managed through `TakeoffStateHandler` (like measurements)
- Surface mesh construction is deferred until a matching scale is found for the page
- Elevation converts to pixels using the page scale ratio: `elevation_px = elevation_real * ratio`
- Same scale ratio applies to elevation as horizontal distances
- Volume output supports both raw (`raw_volume_against`) and unit-aware (`volume_against`) results
- Remove `#[napi]` from core contour types; bindings layer defines its own NAPI input types

## Core Type Changes (`takeoff_core`)

### ContourLineInput / ContourPointOfInterestInput

Add `unit: Unit` field to both types:

```rust
pub struct ContourLineInput {
  pub elevation: f64,    // real-world value (e.g., 5.0)
  pub unit: Unit,        // the unit (e.g., Feet)
  pub points: Vec<Point>,
}

pub struct ContourPointOfInterestInput {
  pub elevation: f64,
  pub unit: Unit,
  pub point: Point,
}
```

Remove `#[napi(object)]` from core contour types. The bindings layer handles NAPI.

### Elevation-to-Pixel Conversion

New method on `ContourInput`:

```rust
impl ContourInput {
  /// Convert contour elevations to pixel values using the given scale.
  /// Handles unit conversion if contour unit differs from scale unit.
  pub fn get_points_with_scale(&self, scale: &Scale) -> TakeoffResult<Vec<Point3D>> { ... }
}
```

Conversion: `elevation_in_scale_unit = Unit::convert(elevation, contour_unit, scale_unit)`, then `elevation_px = elevation_in_scale_unit * scale.ratio()`.

### SurfaceMesh Construction

`to_surface_mesh` now requires a scale:

```rust
impl ContourInput {
  pub fn to_surface_mesh(&self, scale: &Scale) -> TakeoffResult<SurfaceMesh> { ... }
}
```

The `TryFrom<ContourInput>` impl is removed.

## Bindings Layer (`packages/bindings`)

### ContourWrapper

```rust
#[napi]
pub struct ContourWrapper {
  contour: Arc<Mutex<ContourInput>>,
  scale: Arc<Mutex<Option<Scale>>>,
  surface_mesh: Arc<Mutex<Option<SurfaceMesh>>>,
  state: Arc<TakeoffStateHandler>,
}
```

### NAPI Input Types

Separate JS-facing input types with `#[napi(object)]`:

```rust
#[napi(object)]
pub struct ContourLineInputJs {
  pub elevation: f64,
  pub unit: Unit,
  pub points: Vec<Point>,
}

#[napi(object)]
pub struct ContourPointOfInterestInputJs {
  pub elevation: f64,
  pub unit: Unit,
  pub point: Point,
}

#[napi(object)]
pub struct ContourInputJs {
  pub id: String,
  pub name: Option<String>,
  pub page_id: String,
  pub lines: Vec<ContourLineInputJs>,
  pub points_of_interest: Vec<ContourPointOfInterestInputJs>,
}
```

With `From<ContourInputJs> for ContourInput` conversions.

### Scale Resolution

Mirrors `MeasurementWrapper::calculate_scale()`:
1. Look up scales for the contour's `page_id` via `state.get_page_scales()`
2. Try `Scale::Area` first (check bounding box containment)
3. Fall back to `Scale::Default`
4. On success: cache scale, rebuild surface mesh

### Deferred Mesh

If no scale is available, `surface_mesh` stays `None`. Methods `get_scatter_data`, `volume_against`, `get_z_at`, `get_surface_points` return `None`.

### Volume Output

```rust
// Raw pixel-space values
#[napi]
pub fn raw_volume_against(&self, reference: ReferenceSurfaceInput, cell_size: Option<f64>) -> Option<VolumetricResult> { ... }

// Unit-aware values (cubic feet, etc.)
#[napi]
pub fn volume_against(&self, reference: ReferenceSurfaceInput, cell_size: Option<f64>) -> Option<VolumetricUnitResult> { ... }
```

`VolumetricUnitResult` wraps cut/fill as `UnitValue` with `UnitValueItemType::Volume`.

## State Handler Integration

### New DashMap

```rust
pub struct TakeoffStateHandler {
  pages: Arc<DashMap<String, Page>>,
  groups: Arc<DashMap<String, GroupWrapper>>,
  measurements: Arc<DashMap<String, MeasurementWrapper>>,
  scales: Arc<DashMap<String, Scale>>,
  contours: Arc<DashMap<String, ContourWrapper>>,  // NEW
}
```

### CRUD Methods

- `upsert_contour(contour: ContourInputJs) -> Option<ContourInputJs>`
- `remove_contour(contour_id: String) -> Option<ContourInputJs>`
- `get_contour(contour_id: String) -> Option<ContourWrapper>`
- `get_contours_by_page_id(page_id: String) -> Vec<ContourWrapper>`
- `get_contours_missing_scale() -> Vec<ContourWrapper>`

### Scale Change Recomputation

`upsert_scale` and `remove_scale` trigger `compute_contours(page_id)` in addition to existing `compute_page(page_id)`.

### StateOptions

Add `contours: Vec<ContourInput>` to `StateOptions` for bulk initialization.

## Error Handling

New `TakeoffError` variant:

```rust
ContourMissingScale { contour_id: String }
```

Handled softly: contour stores, mesh is `None`, methods return `None`.

## Testing

### Core (Rust)

- `test_contour_input_with_unit` -- unit field constructs correctly
- `test_get_points_with_scale` -- elevation conversion (5.0 ft, 10px/ft scale -> z=50px)
- `test_get_points_with_unit_conversion` -- cross-unit (meters contour, feet scale)
- `test_to_surface_mesh_with_scale` -- end-to-end mesh with scale

### Bindings (Rust)

- `test_contour_wrapper_with_scale` -- resolves scale, builds mesh
- `test_contour_wrapper_no_scale` -- stores contour, mesh None
- `test_contour_wrapper_scale_arrives_later` -- deferred mesh construction
- `test_volume_against_with_units` -- VolumetricUnitResult cubic units
- `test_raw_volume_against` -- raw pixel values

### Integration (TypeScript/Vitest)

- Update `contour.test.ts` to use new input format with `unit` field
- Test deferred scale pattern from JS

## Files to Change

| File | Change |
|------|--------|
| `crates/takeoff_core/src/contour.rs` | Add `unit` to input types, remove `#[napi]`, add `get_points_with_scale`, update `to_surface_mesh` signature |
| `crates/takeoff_core/src/state.rs` | Add `contours` to `StateOptions` |
| `crates/takeoff_core/src/error.rs` | Add `ContourMissingScale` variant |
| `crates/takeoff_core/src/volume.rs` | Add `VolumetricUnitResult` type |
| `packages/bindings/src/contour.rs` | Rewrite ContourWrapper, add JS input types, scale resolution, deferred mesh |
| `packages/bindings/src/state.rs` | Add contours DashMap, CRUD methods, compute_contours |
| `packages/bindings/__test__/contour.test.ts` | Update tests for new input format |
