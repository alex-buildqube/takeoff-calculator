---
name: Volumetric Cut Fill Calculation
overview: Add volumetric cut/fill calculations between a SurfaceMesh (terrain) and a reference surface defined by a geo polygon at a constant elevation. Surface the main design choices, algorithm options, and edge cases.
todos: []
isProject: false
---

# Volumetric Cut/Fill with Polygon Reference Surface

## Context

The reference surface is a **geo polygon with a constant elevation** (e.g. foundation footprint at z = 5729), not an infinite horizontal plane. Volume is computed only within the polygon boundary.

- **Cut**: Terrain above the reference → dirt to remove
- **Fill**: Terrain below the reference → dirt to bring in

## Design Considerations

### 1. Polygon Outside Mesh

If the polygon extends beyond the contour mesh coverage, `SurfaceMesh::z_at(x, y)` returns `None`. Options:

- **Fail** or return `Result` with an error describing uncovered cells
- **Partial result** with a flag/count of cells that had no terrain data
- **Require** the caller to ensure the polygon is fully inside mesh bounds

Recommendation: Return a `VolumetricResult` that includes `cut`, `fill`, and `uncovered_area` (or `uncovered_cell_count`). Callers can decide whether to fail or warn.

### 2. Algorithm Choice

| Approach              | Pros                                                                                     | Cons                                                                                                       |
| --------------------- | ---------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| **Grid sampling**     | Simple, reuses `z_at`, handles any polygon (including holes via `geo::Contains`), robust | Approximate; accuracy depends on resolution                                                                |
| **Triangle clipping** | Exact (within FP)                                                                        | Needs polygon–polygon intersection (geo-clipper or similar), complex clipping of mesh triangles to polygon |

**Recommendation**: Start with **grid sampling**. Add a configurable `resolution` (cell size or grid count). Document that results are approximate. Triangle clipping can be a later optimization if exactness is required.

### 3. Volume Formula (Grid Sampling)

For each grid cell whose center `(x, y)` is inside the polygon:

- `terrain_z = mesh.z_at(x, y)`
- If `None`: count as uncovered, skip
- `delta = terrain_z - ref_elevation`
- If `delta > 0`: add `cell_area * delta` to cut
- If `delta < 0`: add `cell_area * (-delta)` to fill

Sum over all covered cells.

### 4. Holes in Polygon

`geo::Polygon` supports interior rings (holes). Use `polygon.contains(&point)` — it correctly excludes points inside holes. No extra logic needed if using `Contains`.

### 5. Input Shape

Define a reference surface type:

```rust
pub struct ReferenceSurface {
  pub polygon: geo::Polygon<f64>,  // 2D boundary
  pub elevation: f64,               // constant z for the whole polygon
}
```

The polygon can come from `Vec<Point>` (exterior ring) and optional holes, matching the existing `Measurement::to_polygon()` pattern in [measurement.rs](crates/takeoff_core/src/measurement.rs).

### 6. Coordinate System

Ensure the polygon and `SurfaceMesh` use the same (x, y) coordinate system. The contour data and polygon (e.g. from a measurement or plan) must be in the same units and projection.

### 7. Units

The codebase uses "pixels" in some comments (e.g. ContourLineInput). For earthwork, elevations are typically in feet or meters. The volumetric result should be in **length^3** (cubic feet, cubic meters). If coordinates are in real-world units, `cell_area * delta` gives volume in those units. Consider returning raw values and letting the caller convert via `uom` if needed.

## Implementation Plan

### Step 1: Reference surface type

Add `ReferenceSurface` (or `ElevatedPolygon`) in a new module `volume` or in [contour.rs](crates/takeoff_core/src/contour.rs):

- `polygon: geo::Polygon<f64>`
- `elevation: f64`
- Constructor from `Vec<Point>` + elevation (or from `geo::Polygon`)

### Step 2: Volumetric result type

```rust
pub struct VolumetricResult {
  pub cut: f64,           // volume to remove
  pub fill: f64,          // volume to add
  pub uncovered_area: f64, // area where z_at returned None
}
```

### Step 3: Grid-based volume computation

Add `SurfaceMesh::volume_against(&self, reference: &ReferenceSurface, cell_size: f64) -> VolumetricResult`:

- Compute polygon bounding box
- Generate grid of cell centers with spacing `cell_size`
- For each center: if `polygon.contains(center)`, call `z_at`, accumulate cut/fill or uncovered
- Use `geo::Contains` and `Coord` for point-in-polygon

### Step 4: Tests

- Rectangle polygon fully inside mesh, flat terrain vs reference → known cut/fill
- Polygon with hole
- Polygon partially outside mesh → verify `uncovered_area` is non-zero

### Step 5: Edge cases

- Empty polygon → return zeros
- Polygon fully outside mesh → all uncovered
- `cell_size` too large (e.g. bigger than polygon) → at least one cell at centroid

## Files to Touch

- New module `crates/takeoff_core/src/volume.rs`
- [lib.rs](crates/takeoff_core/src/lib.rs): export `volume` module
- [coords.rs](crates/takeoff_core/src/coords.rs): `Point` already converts to `Coord` for geo

## Open Questions

1. **Default cell size**: Should it be derived from mesh bounds (e.g. sqrt(area/1000)) or always passed explicitly?
1. Let's have the default be sqrt(area/1000) and have the option to define custom cell size
1. **NAPI exposure**: Should `volume_against` and `ReferenceSurface` be exposed to TypeScript for the sample app?
1. Yes
