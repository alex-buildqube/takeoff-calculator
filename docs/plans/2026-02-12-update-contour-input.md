# Update Contour Input Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Change contour elevation from raw pixel values to unit-based real-world measurements, integrate contours into the state handler, and defer surface mesh construction until a page scale is available.

**Architecture:** Mirror the `MeasurementWrapper` pattern: core types hold elevation + unit, bindings layer manages scale lookup and mesh lifecycle via `Arc<Mutex<>>` fields. State handler gets a `contours` DashMap with CRUD methods and scale-change recomputation.

**Tech Stack:** Rust (takeoff_core, napi-rs bindings), TypeScript (Vitest), `uom` crate for unit conversions, `delaunator` for triangulation, `DashMap` for concurrent state.

**Design doc:** `docs/plans/2026-02-12-update-contour-input-design.md`

---

### Task 1: Add `unit` field to core contour types and remove `#[napi]`

**Files:**
- Modify: `crates/takeoff_core/src/contour.rs:1-68`

**Step 1: Update imports and remove `napi_derive`**

In `crates/takeoff_core/src/contour.rs`, replace the imports block:

```rust
// Old (line 1-9):
use crate::{
  TakeoffError,
  coords::{Point, Point3D},
  error::TakeoffResult,
};
use delaunator::triangulate;
use geo::{BoundingRect, Geometry, GeometryCollection, LineString, Point as GeoPoint};
use napi_derive::napi;
use serde::{Deserialize, Serialize};

// New:
use crate::{
  TakeoffError,
  coords::{Point, Point3D},
  error::TakeoffResult,
  scale::Scale,
  unit::Unit,
};
use delaunator::triangulate;
use geo::{BoundingRect, Geometry, GeometryCollection, LineString, Point as GeoPoint};
use serde::{Deserialize, Serialize};
```

**Step 2: Update `ContourLineInput`**

Remove `#[napi(object)]` and add `unit: Unit`:

```rust
// Old (line 11-17):
#[napi(object)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContourLineInput {
  /// The elevation of the contour line (in pixels)
  pub elevation: f64,
  pub points: Vec<Point>,
}

// New:
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContourLineInput {
  /// The elevation of the contour line (real-world value)
  pub elevation: f64,
  /// The unit of the elevation value
  pub unit: Unit,
  pub points: Vec<Point>,
}
```

**Step 3: Update `ContourPointOfInterestInput`**

Remove `#[napi(object)]` and add `unit: Unit`:

```rust
// Old (line 39-45):
#[napi(object)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContourPointOfInterestInput {
  /// The elevation of the point of interest (in pixels)
  pub elevation: f64,
  pub point: Point,
}

// New:
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContourPointOfInterestInput {
  /// The elevation of the point of interest (real-world value)
  pub elevation: f64,
  /// The unit of the elevation value
  pub unit: Unit,
  pub point: Point,
}
```

**Step 4: Update `ContourInput`**

Remove `#[napi(object)]`:

```rust
// Old (line 57-68):
#[napi(object)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContourInput {
  ...
}

// New:
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContourInput {
  pub id: String,
  pub name: Option<String>,
  pub page_id: String,
  pub lines: Vec<ContourLineInput>,
  pub points_of_interest: Vec<ContourPointOfInterestInput>,
}
```

**Step 5: Update existing `From<ContourLineInput> for Vec<Point3D>`**

This impl currently uses `input.elevation` directly as pixel z. It now needs a scale to convert. **Remove this impl** — it will be replaced by `get_points_with_scale` in Task 2.

Remove lines 25-37:
```rust
// DELETE this entire impl block:
impl From<ContourLineInput> for Vec<Point3D> { ... }
```

**Step 6: Update existing `From<ContourPointOfInterestInput> for Point3D`**

Same issue — remove this impl:

```rust
// DELETE this entire impl block:
impl From<ContourPointOfInterestInput> for Point3D { ... }
```

**Step 7: Update `get_points` to compile**

The `get_points` method (line 191-203) uses the removed `From` impls. Update it to keep compiling but mark it as pixel-only (it will be replaced in Task 2):

```rust
// Old:
pub fn get_points(&self) -> Vec<Point3D> {
  let mut points: Vec<Point3D> = Vec::new();
  for line in self.lines.clone() {
    let line_points: Vec<Point3D> = line.into();
    points.extend(line_points);
  }
  for point_of_interest in self.points_of_interest.clone() {
    let point_of_interest_point: Point3D = point_of_interest.into();
    points.push(point_of_interest_point);
  }
  points
}

// New (temporary — uses elevation as raw value, no unit conversion):
pub fn get_points_raw(&self) -> Vec<Point3D> {
  let mut points: Vec<Point3D> = Vec::new();
  for line in &self.lines {
    for p in &line.points {
      points.push(Point3D::new(p.x, p.y, line.elevation));
    }
  }
  for poi in &self.points_of_interest {
    points.push(Point3D::new(poi.point.x, poi.point.y, poi.elevation));
  }
  points
}
```

**Step 8: Update `to_surface_mesh` and `TryFrom` to use `get_points_raw` temporarily**

In the `TryFrom<ContourInput> for SurfaceMesh` impl (line 142-178), change `input.get_points()` to `input.get_points_raw()`. This is temporary — Task 3 replaces this entirely.

**Step 9: Update all tests in this file to include `unit` field**

Every test that constructs `ContourLineInput` or `ContourPointOfInterestInput` needs `unit: Unit::Feet` (or any unit — exact value doesn't matter for raw tests).

Example — `test_surface_mesh_success` (line 242-271):
```rust
// Add to each ContourLineInput:
ContourLineInput {
  elevation: 10.0,
  unit: Unit::Feet,  // ADD THIS
  points: vec![...],
}

// Add to each ContourPointOfInterestInput:
ContourPointOfInterestInput {
  elevation: 5.0,
  unit: Unit::Feet,  // ADD THIS
  point: Point::new(5.0, 5.0),
}
```

Apply this to ALL test functions: `test_surface_mesh_success`, `test_surface_mesh_too_few_points`, `test_surface_mesh_collinear`, `test_surface_mesh_deduplication`, `test_z_at_at_vertex`.

**Step 10: Run core tests to verify everything compiles**

Run: `cargo test -p takeoff_core`
Expected: All existing tests pass (they use raw elevation values, no scale needed yet)

**Step 11: Commit**

```bash
git add crates/takeoff_core/src/contour.rs
git commit -m "feat: add unit field to contour elevation types and remove napi from core"
```

---

### Task 2: Add `get_points_with_scale` method

**Files:**
- Modify: `crates/takeoff_core/src/contour.rs`

**Step 1: Write the failing test for same-unit elevation conversion**

Add to the `#[cfg(test)]` module in `crates/takeoff_core/src/contour.rs`:

```rust
use crate::scale::{Scale, ScaleDefinition};
use crate::unit::Unit;

#[test]
fn test_get_points_with_scale() {
  let input = ContourInput {
    id: "1".to_string(),
    name: None,
    page_id: "1".to_string(),
    lines: vec![ContourLineInput {
      elevation: 5.0,
      unit: Unit::Feet,
      points: vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0)],
    }],
    points_of_interest: vec![ContourPointOfInterestInput {
      elevation: 3.0,
      unit: Unit::Feet,
      point: Point::new(5.0, 5.0),
    }],
  };
  let scale = Scale::Default {
    id: "s1".to_string(),
    page_id: "1".to_string(),
    scale: ScaleDefinition {
      pixel_distance: 100.0,
      real_distance: 10.0,
      unit: Unit::Feet,
    },
  };
  // ratio = 100/10 = 10 px/ft
  // 5.0 ft * 10 = 50.0 px, 3.0 ft * 10 = 30.0 px
  let points = input.get_points_with_scale(&scale).unwrap();
  assert_eq!(points.len(), 3);
  assert!((points[0].z - 50.0).abs() < 1e-6);
  assert!((points[1].z - 50.0).abs() < 1e-6);
  assert!((points[2].z - 30.0).abs() < 1e-6);
  // x/y should be unchanged (still pixels)
  assert!((points[0].x - 0.0).abs() < 1e-6);
  assert!((points[1].x - 10.0).abs() < 1e-6);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p takeoff_core test_get_points_with_scale`
Expected: FAIL — method `get_points_with_scale` does not exist

**Step 3: Implement `get_points_with_scale`**

Add to the `impl ContourInput` block in `crates/takeoff_core/src/contour.rs`:

```rust
/// Convert contour elevations to pixel values using the given scale.
///
/// Each contour line/POI elevation is converted from its unit to the scale's unit,
/// then multiplied by the scale ratio (pixels per real-world unit).
///
/// x/y coordinates are already in pixels and remain unchanged.
pub fn get_points_with_scale(&self, scale: &Scale) -> TakeoffResult<Vec<Point3D>> {
  let ratio = scale.ratio()?;
  let scale_unit = scale.get_unit();
  let mut points: Vec<Point3D> = Vec::new();

  for line in &self.lines {
    let elevation_in_scale_unit = if line.unit == scale_unit {
      line.elevation
    } else {
      line.unit.convert(line.elevation as f32, &scale_unit) as f64
    };
    let elevation_px = elevation_in_scale_unit * ratio;
    for p in &line.points {
      points.push(Point3D::new(p.x, p.y, elevation_px));
    }
  }

  for poi in &self.points_of_interest {
    let elevation_in_scale_unit = if poi.unit == scale_unit {
      poi.elevation
    } else {
      poi.unit.convert(poi.elevation as f32, &scale_unit) as f64
    };
    let elevation_px = elevation_in_scale_unit * ratio;
    points.push(Point3D::new(poi.point.x, poi.point.y, elevation_px));
  }

  Ok(points)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p takeoff_core test_get_points_with_scale`
Expected: PASS

**Step 5: Write test for cross-unit conversion**

```rust
#[test]
fn test_get_points_with_scale_cross_unit() {
  let input = ContourInput {
    id: "1".to_string(),
    name: None,
    page_id: "1".to_string(),
    lines: vec![ContourLineInput {
      elevation: 1.0,
      unit: Unit::Meters,
      points: vec![Point::new(0.0, 0.0)],
    }],
    points_of_interest: vec![],
  };
  let scale = Scale::Default {
    id: "s1".to_string(),
    page_id: "1".to_string(),
    scale: ScaleDefinition {
      pixel_distance: 120.0,
      real_distance: 1.0,
      unit: Unit::Feet,
    },
  };
  // 1 meter = ~3.28084 feet
  // ratio = 120/1 = 120 px/ft
  // 3.28084 * 120 = ~393.7 px
  let points = input.get_points_with_scale(&scale).unwrap();
  assert_eq!(points.len(), 1);
  let expected_z = Unit::Meters.convert(1.0, &Unit::Feet) as f64 * 120.0;
  assert!(
    (points[0].z - expected_z).abs() < 1.0,
    "expected ~{}, got {}",
    expected_z,
    points[0].z
  );
}
```

**Step 6: Run test to verify it passes**

Run: `cargo test -p takeoff_core test_get_points_with_scale_cross_unit`
Expected: PASS

**Step 7: Commit**

```bash
git add crates/takeoff_core/src/contour.rs
git commit -m "feat: add get_points_with_scale for elevation-to-pixel conversion"
```

---

### Task 3: Update `to_surface_mesh` to require a `Scale`

**Files:**
- Modify: `crates/takeoff_core/src/contour.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_to_surface_mesh_with_scale() {
  let input = ContourInput {
    id: "1".to_string(),
    name: Some("test".to_string()),
    page_id: "1".to_string(),
    lines: vec![ContourLineInput {
      elevation: 10.0,
      unit: Unit::Feet,
      points: vec![
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        Point::new(10.0, 10.0),
        Point::new(0.0, 10.0),
      ],
    }],
    points_of_interest: vec![ContourPointOfInterestInput {
      elevation: 5.0,
      unit: Unit::Feet,
      point: Point::new(5.0, 5.0),
    }],
  };
  let scale = Scale::Default {
    id: "s1".to_string(),
    page_id: "1".to_string(),
    scale: ScaleDefinition {
      pixel_distance: 10.0,
      real_distance: 1.0,
      unit: Unit::Feet,
    },
  };
  // ratio = 10px/ft. 10ft * 10 = 100px, 5ft * 10 = 50px
  let mesh = input.to_surface_mesh(&scale).unwrap();
  assert_eq!(mesh.vertices.len(), 5);
  assert!(!mesh.triangles.is_empty());
  // Check that z values are in pixels
  let corner_z = mesh.vertices.iter().find(|v| v.x == 0.0 && v.y == 0.0).unwrap().z;
  assert!((corner_z - 100.0).abs() < 1e-6);
  let center_z = mesh.vertices.iter().find(|v| (v.x - 5.0).abs() < 1e-6 && (v.y - 5.0).abs() < 1e-6).unwrap().z;
  assert!((center_z - 50.0).abs() < 1e-6);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p takeoff_core test_to_surface_mesh_with_scale`
Expected: FAIL — `to_surface_mesh` doesn't accept a `&Scale` parameter

**Step 3: Update `to_surface_mesh` signature and implementation**

Change `to_surface_mesh` to accept `&Scale`:

```rust
impl ContourInput {
  /// Convert contour input to a triangulated 3D surface mesh.
  /// Elevations are converted from real-world units to pixels using the scale.
  ///
  /// # Errors
  ///
  /// Returns [`TakeoffError::SurfaceMeshTooFewPoints`] if there are fewer than 3 points.
  /// Returns [`TakeoffError::SurfaceMeshCollinearPoints`] if all points are collinear.
  pub fn to_surface_mesh(&self, scale: &Scale) -> TakeoffResult<SurfaceMesh> {
    let points = self.get_points_with_scale(scale)?;
    let vertices = SurfaceMesh::deduplicate_points(&points);

    if vertices.len() < 3 {
      return Err(TakeoffError::SurfaceMeshTooFewPoints {
        count: vertices.len(),
      });
    }

    let delaunator_points: Vec<delaunator::Point> = vertices
      .iter()
      .map(|p| delaunator::Point { x: p.x, y: p.y })
      .collect();

    let result = triangulate(&delaunator_points);

    if result.triangles.is_empty() {
      return Err(TakeoffError::SurfaceMeshCollinearPoints);
    }

    let triangles: Vec<[u32; 3]> = result
      .triangles
      .chunks_exact(3)
      .map(|chunk| [chunk[0] as u32, chunk[1] as u32, chunk[2] as u32])
      .collect();

    Ok(SurfaceMesh { vertices, triangles })
  }
}
```

**Step 4: Remove `TryFrom<ContourInput> for SurfaceMesh`**

Delete the entire `impl TryFrom<ContourInput> for SurfaceMesh` block (lines 142-178). The `to_surface_mesh` method now replaces it.

**Step 5: Remove `get_points_raw` (no longer needed)**

Delete the `get_points_raw` method added in Task 1. Keep the original `get_points` method name but delete it too — it's replaced by `get_points_with_scale`. Actually, check if `get_points` is used elsewhere first.

Check usage: `get_points` is only called from `TryFrom` which we just removed. Remove `get_points_raw`.

**Step 6: Update existing tests that used `to_surface_mesh()` without scale**

All existing tests calling `input.to_surface_mesh()` need to pass a scale. Create a helper:

```rust
#[cfg(test)]
mod tests {
  use super::*;
  use crate::scale::{Scale, ScaleDefinition};
  use crate::unit::Unit;

  /// 1:1 scale (1 pixel = 1 unit) for tests where scale doesn't matter
  fn identity_scale() -> Scale {
    Scale::Default {
      id: "test-scale".to_string(),
      page_id: "1".to_string(),
      scale: ScaleDefinition {
        pixel_distance: 1.0,
        real_distance: 1.0,
        unit: Unit::Feet,
      },
    }
  }

  // ... existing tests updated to use identity_scale()
}
```

Update each test:
- `test_surface_mesh_success`: `input.to_surface_mesh(&identity_scale())`
- `test_surface_mesh_too_few_points`: same
- `test_surface_mesh_collinear`: same
- `test_surface_mesh_deduplication`: same
- `test_z_at_at_vertex`: same

**Step 7: Run all core tests**

Run: `cargo test -p takeoff_core`
Expected: All tests pass

**Step 8: Commit**

```bash
git add crates/takeoff_core/src/contour.rs
git commit -m "feat: require scale for surface mesh construction"
```

---

### Task 4: Add `ContourMissingScale` error variant and update `StateOptions`

**Files:**
- Modify: `crates/takeoff_core/src/error.rs:12-75`
- Modify: `crates/takeoff_core/src/state.rs`

**Step 1: Add error variant**

In `crates/takeoff_core/src/error.rs`, add after `SurfaceMeshCollinearPoints` (line 56):

```rust
/// No scale found for the contour's page.
#[error("no scale found for contour {contour_id} on page")]
ContourMissingScale { contour_id: String },
```

**Step 2: Add NAPI conversion for new variant**

In the `From<TakeoffError> for NapiError` impl, add before the `PoisonError` arm:

```rust
TakeoffError::ContourMissingScale { .. } => {
  NapiError::new(Status::InvalidArg, error.to_string())
}
```

**Step 3: Add constructor helper**

```rust
impl TakeoffError {
  // ... existing helpers

  /// Create a `ContourMissingScale` error.
  pub fn contour_missing_scale(contour_id: impl Into<String>) -> Self {
    Self::ContourMissingScale {
      contour_id: contour_id.into(),
    }
  }
}
```

**Step 4: Update `StateOptions` in core**

In `crates/takeoff_core/src/state.rs`, add contours:

```rust
use crate::contour::ContourInput;

#[napi(object)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateOptions {
  pub pages: Vec<Page>,
  pub groups: Vec<Group>,
  pub measurements: Vec<Measurement>,
  pub scales: Vec<Scale>,
  pub contours: Vec<ContourInput>,  // NEW
}
```

Note: `ContourInput` no longer has `#[napi(object)]`, so `StateOptions` needs `ContourInput` to NOT be directly used as NAPI. Since `StateOptions` IS an napi(object), we need `ContourInput` to implement the napi `FromNapiValue` trait. **Alternative:** Keep `contours` as a separate parameter in the bindings state handler constructor, not in `StateOptions`. This avoids the circular dependency.

**Decision:** Remove `contours` from core `StateOptions`. Instead, handle contour initialization in the bindings `TakeoffStateHandler::new()` with a separate parameter. This keeps core clean.

**Step 5: Run tests**

Run: `cargo test -p takeoff_core`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/takeoff_core/src/error.rs
git commit -m "feat: add ContourMissingScale error variant"
```

---

### Task 5: Create NAPI input types in bindings

**Files:**
- Modify: `packages/bindings/src/contour.rs`

**Step 1: Add NAPI input type structs**

At the top of `packages/bindings/src/contour.rs`, add the JS-facing input types:

```rust
use napi_derive::napi;
use takeoff_core::contour::{ContourInput, ContourLineInput, ContourPointOfInterestInput};
use takeoff_core::coords::Point;
use takeoff_core::unit::Unit;

#[napi(object)]
#[derive(Debug, Clone)]
pub struct ContourLineInputJs {
  pub elevation: f64,
  pub unit: Unit,
  pub points: Vec<Point>,
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct ContourPointOfInterestInputJs {
  pub elevation: f64,
  pub unit: Unit,
  pub point: Point,
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct ContourInputJs {
  pub id: String,
  pub name: Option<String>,
  pub page_id: String,
  pub lines: Vec<ContourLineInputJs>,
  pub points_of_interest: Vec<ContourPointOfInterestInputJs>,
}
```

**Step 2: Implement `From` conversions**

```rust
impl From<ContourLineInputJs> for ContourLineInput {
  fn from(js: ContourLineInputJs) -> Self {
    Self {
      elevation: js.elevation,
      unit: js.unit,
      points: js.points,
    }
  }
}

impl From<ContourPointOfInterestInputJs> for ContourPointOfInterestInput {
  fn from(js: ContourPointOfInterestInputJs) -> Self {
    Self {
      elevation: js.elevation,
      unit: js.unit,
      point: js.point,
    }
  }
}

impl From<ContourInputJs> for ContourInput {
  fn from(js: ContourInputJs) -> Self {
    Self {
      id: js.id,
      name: js.name,
      page_id: js.page_id,
      lines: js.lines.into_iter().map(|l| l.into()).collect(),
      points_of_interest: js.points_of_interest.into_iter().map(|p| p.into()).collect(),
    }
  }
}
```

**Step 3: Verify it compiles**

Run: `cargo check -p takeoff-calculator`
Expected: Compiles (there will be warnings about unused code until Task 6)

**Step 4: Commit**

```bash
git add packages/bindings/src/contour.rs
git commit -m "feat: add NAPI input types for contour with unit field"
```

---

### Task 6: Rewrite `ContourWrapper` with scale-aware lifecycle

**Files:**
- Modify: `packages/bindings/src/contour.rs`

**Step 1: Write failing test for ContourWrapper with scale**

Add to `#[cfg(test)]` module in `packages/bindings/src/contour.rs`:

```rust
#[cfg(test)]
mod tests {
  use super::*;
  use takeoff_core::contour::ContourLineInput;
  use takeoff_core::coords::Point;
  use takeoff_core::scale::{Scale, ScaleDefinition};
  use takeoff_core::unit::Unit;

  fn test_contour_input() -> ContourInput {
    ContourInput {
      id: "c1".to_string(),
      name: None,
      page_id: "p1".to_string(),
      lines: vec![ContourLineInput {
        elevation: 10.0,
        unit: Unit::Feet,
        points: vec![
          Point::new(0.0, 0.0),
          Point::new(100.0, 0.0),
          Point::new(100.0, 100.0),
          Point::new(0.0, 100.0),
        ],
      }],
      points_of_interest: vec![],
    }
  }

  fn test_scale() -> Scale {
    Scale::Default {
      id: "s1".to_string(),
      page_id: "p1".to_string(),
      scale: ScaleDefinition {
        pixel_distance: 1.0,
        real_distance: 1.0,
        unit: Unit::Feet,
      },
    }
  }

  #[test]
  fn test_contour_wrapper_no_scale() {
    let wrapper = ContourWrapper::from_input(test_contour_input(), Arc::new(TakeoffStateHandler::default()));
    assert!(wrapper.get_surface_points().is_none());
    assert!(wrapper.get_scatter_data(10).is_none());
  }

  #[test]
  fn test_contour_wrapper_with_scale() {
    let state = TakeoffStateHandler::default();
    let wrapper = ContourWrapper::from_input(test_contour_input(), Arc::new(state));
    wrapper.set_scale(test_scale());
    let points = wrapper.get_surface_points();
    assert!(points.is_some());
    let points = points.unwrap();
    assert_eq!(points.len(), 4);
    // With 1:1 scale, elevation 10.0 ft = 10.0 px
    assert!((points[0].z - 10.0).abs() < 1e-6);
  }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p takeoff-calculator test_contour_wrapper`
Expected: FAIL — `from_input` method doesn't exist

**Step 3: Rewrite `ContourWrapper`**

Replace the entire `ContourWrapper` struct and impl in `packages/bindings/src/contour.rs`:

```rust
use crate::state::TakeoffStateHandler;
use crate::utils::lock_mutex;
use anyhow::Result;
use napi_derive::napi;
use std::sync::{Arc, Mutex};
use takeoff_core::contour::{ContourInput, SurfaceMesh};
use takeoff_core::coords::Point3D;
use takeoff_core::error::TakeoffResult;
use takeoff_core::scale::Scale;
use takeoff_core::unit::{UnitValue, UnitValueItemType};
use takeoff_core::volume::{ReferenceSurface, ReferenceSurfaceInput, VolumetricResult};

#[napi]
#[derive(Debug, Clone)]
pub struct ContourWrapper {
  contour: Arc<Mutex<ContourInput>>,
  scale: Arc<Mutex<Option<Scale>>>,
  surface_mesh: Arc<Mutex<Option<SurfaceMesh>>>,
  state: Arc<TakeoffStateHandler>,
}

#[napi]
impl ContourWrapper {
  pub fn from_input(contour: ContourInput, state: Arc<TakeoffStateHandler>) -> Self {
    Self {
      contour: Arc::new(Mutex::new(contour)),
      scale: Arc::new(Mutex::new(None)),
      surface_mesh: Arc::new(Mutex::new(None)),
      state,
    }
  }

  #[napi(constructor)]
  pub fn new(contour: ContourInputJs) -> Self {
    let input: ContourInput = contour.into();
    Self::from_input(input, Arc::new(TakeoffStateHandler::default()))
  }

  pub fn set_contour(&self, contour: ContourInput) {
    *lock_mutex(self.contour.lock(), "contour")
      .expect("BUG: contour mutex should not be poisoned") = contour;
    let _ = self.rebuild_surface_mesh();
  }

  pub fn set_scale(&self, scale: Scale) {
    *lock_mutex(self.scale.lock(), "scale")
      .expect("BUG: scale mutex should not be poisoned") = Some(scale);
    let _ = self.rebuild_surface_mesh();
  }

  pub fn calculate_scale(&self) -> Option<Scale> {
    let mut current_scale: Option<Scale> = None;
    let contour = lock_mutex(self.contour.lock(), "contour").ok()?;
    let geometry = contour.bounding_box().map(|((min_x, min_y), (max_x, max_y))| {
      use geo::{Coord, Rect};
      geo::Geometry::Rect(Rect::new(
        Coord { x: min_x, y: min_y },
        Coord { x: max_x, y: max_y },
      ))
    })?;
    let page_id = contour.page_id.clone();
    drop(contour);

    for scale in self.state.get_page_scales(&page_id) {
      if matches!(scale, Scale::Area { .. }) {
        if scale.is_in_bounding_box(&geometry) {
          self.set_scale(scale.clone());
          return Some(scale);
        }
      } else {
        current_scale = Some(scale.clone());
      }
    }
    if let Some(scale) = current_scale {
      self.set_scale(scale.clone());
      return Some(scale);
    }
    None
  }

  fn rebuild_surface_mesh(&self) -> TakeoffResult<()> {
    let scale_guard = lock_mutex(self.scale.lock(), "scale")?;
    if let Some(scale) = scale_guard.as_ref() {
      let contour = lock_mutex(self.contour.lock(), "contour")?;
      match contour.to_surface_mesh(scale) {
        Ok(mesh) => {
          drop(contour);
          drop(scale_guard);
          *lock_mutex(self.surface_mesh.lock(), "surface_mesh")? = Some(mesh);
        }
        Err(_) => {
          drop(contour);
          drop(scale_guard);
          *lock_mutex(self.surface_mesh.lock(), "surface_mesh")? = None;
        }
      }
    }
    Ok(())
  }

  #[napi(getter)]
  pub fn id(&self) -> String {
    lock_mutex(self.contour.lock(), "contour")
      .expect("BUG: contour mutex should not be poisoned")
      .id
      .clone()
  }

  #[napi(getter)]
  pub fn page_id(&self) -> String {
    lock_mutex(self.contour.lock(), "contour")
      .expect("BUG: contour mutex should not be poisoned")
      .page_id
      .clone()
  }

  #[napi(getter)]
  pub fn get_scale(&self) -> Option<Scale> {
    lock_mutex(self.scale.lock(), "scale")
      .ok()
      .and_then(|s| s.clone())
  }

  #[napi]
  pub fn get_surface_points(&self) -> Option<Vec<Point3D>> {
    let mesh_guard = lock_mutex(self.surface_mesh.lock(), "surface_mesh").ok()?;
    mesh_guard.as_ref().map(|mesh| mesh.vertices.clone())
  }

  #[napi]
  pub fn get_z_at(&self, x: f64, y: f64) -> Option<f64> {
    let mesh_guard = lock_mutex(self.surface_mesh.lock(), "surface_mesh").ok()?;
    let mesh = mesh_guard.as_ref()?;
    mesh.z_at(x, y)
  }

  #[napi]
  pub fn get_scatter_data(&self, step: i32) -> Option<Vec<Point3D>> {
    if step <= 0 {
      return None;
    }
    let step = step as usize;

    let contour = lock_mutex(self.contour.lock(), "contour").ok()?;
    let bounding_box = contour.bounding_box()?;
    drop(contour);

    let mesh_guard = lock_mutex(self.surface_mesh.lock(), "surface_mesh").ok()?;
    let surface_mesh = mesh_guard.as_ref()?;

    let (min_x, min_y) = bounding_box.0;
    let (max_x, max_y) = bounding_box.1;
    let mut data: Vec<Point3D> = Vec::new();

    let x_start = min_x.floor() as i32;
    let x_end = max_x.ceil() as i32;
    let y_start = min_y.floor() as i32;
    let y_end = max_y.ceil() as i32;

    for x in (x_start..=x_end).step_by(step) {
      for y in (y_start..=y_end).step_by(step) {
        if let Some(z) = surface_mesh.z_at(x as f64, y as f64) {
          data.push(Point3D::new(x as f64, y as f64, z));
        }
      }
    }
    Some(data)
  }

  /// Compute raw cut/fill volume (pixel-space values) against a reference surface.
  #[napi]
  pub fn raw_volume_against(
    &self,
    reference: ReferenceSurfaceInput,
    cell_size: Option<f64>,
  ) -> Option<VolumetricResult> {
    let mesh_guard = lock_mutex(self.surface_mesh.lock(), "surface_mesh").ok()?;
    let mesh = mesh_guard.as_ref()?;
    let reference = ReferenceSurface::from(reference);
    Some(mesh.volume_against(&reference, cell_size))
  }

  /// Compute unit-aware cut/fill volume against a reference surface.
  /// Returns None if surface mesh or scale is not available.
  #[napi]
  pub fn volume_against(
    &self,
    reference: ReferenceSurfaceInput,
    cell_size: Option<f64>,
  ) -> Option<VolumetricUnitResult> {
    let mesh_guard = lock_mutex(self.surface_mesh.lock(), "surface_mesh").ok()?;
    let mesh = mesh_guard.as_ref()?;
    let scale_guard = lock_mutex(self.scale.lock(), "scale").ok()?;
    let scale = scale_guard.as_ref()?;

    let reference_surface = ReferenceSurface::from(reference);
    let raw = mesh.volume_against(&reference_surface, cell_size);

    let ratio = scale.ratio().ok()?;
    let unit = scale.get_unit();

    // Raw volume is in cubic pixels. Convert: real_volume = raw_volume / ratio^3
    let ratio_cubed = ratio * ratio * ratio;
    let cut_real = raw.cut / ratio_cubed;
    let fill_real = raw.fill / ratio_cubed;
    let uncovered_area_real = raw.uncovered_area / (ratio * ratio);

    Some(VolumetricUnitResult {
      cut: UnitValue::from_volume(unit.get_volume_unit(cut_real as f32)),
      fill: UnitValue::from_volume(unit.get_volume_unit(fill_real as f32)),
      uncovered_area: UnitValue::from_area(unit.get_area_unit(uncovered_area_real as f32)),
    })
  }
}
```

**Step 4: Add `VolumetricUnitResult` struct**

Add in `packages/bindings/src/contour.rs` (this is a bindings-layer type, not core):

```rust
#[napi(object)]
#[derive(Debug, Clone)]
pub struct VolumetricUnitResult {
  pub cut: UnitValue,
  pub fill: UnitValue,
  pub uncovered_area: UnitValue,
}
```

**Step 5: Run tests**

Run: `cargo test -p takeoff-calculator test_contour_wrapper`
Expected: PASS

**Step 6: Commit**

```bash
git add packages/bindings/src/contour.rs
git commit -m "feat: rewrite ContourWrapper with scale-aware lifecycle and unit volume output"
```

---

### Task 7: Integrate contours into `TakeoffStateHandler`

**Files:**
- Modify: `packages/bindings/src/state.rs`

**Step 1: Write failing test for state handler contour management**

Add to `#[cfg(test)]` module in `packages/bindings/src/state.rs`:

```rust
use crate::contour::{ContourInputJs, ContourLineInputJs};
use takeoff_core::unit::Unit;

#[test]
fn test_upsert_contour_with_deferred_scale() {
  let state = TakeoffStateHandler::new(None);
  state.upsert_contour(ContourInputJs {
    id: "c1".to_string(),
    name: None,
    page_id: "1".to_string(),
    lines: vec![ContourLineInputJs {
      elevation: 10.0,
      unit: Unit::Feet,
      points: vec![
        Point::new(0.0, 0.0),
        Point::new(100.0, 0.0),
        Point::new(100.0, 100.0),
        Point::new(0.0, 100.0),
      ],
    }],
    points_of_interest: vec![],
  });

  // No scale yet — mesh should be None
  let contour = state.get_contour("c1".to_string()).unwrap();
  assert!(contour.get_surface_points().is_none());

  // Add a scale for the page
  state.upsert_scale(Scale::Default {
    id: "s1".to_string(),
    page_id: "1".to_string(),
    scale: ScaleDefinition {
      pixel_distance: 1.0,
      real_distance: 1.0,
      unit: Unit::Feet,
    },
  });

  // Now mesh should be available
  let contour = state.get_contour("c1".to_string()).unwrap();
  assert!(contour.get_surface_points().is_some());
  assert_eq!(contour.get_surface_points().unwrap().len(), 4);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p takeoff-calculator test_upsert_contour_with_deferred_scale`
Expected: FAIL — `upsert_contour` and related methods don't exist

**Step 3: Add contours DashMap to state handler**

In `packages/bindings/src/state.rs`, add to the struct:

```rust
use crate::contour::{ContourInputJs, ContourWrapper};

#[napi]
#[derive(Debug, Clone, Default)]
pub struct TakeoffStateHandler {
  pages: Arc<DashMap<String, Page>>,
  groups: Arc<DashMap<String, GroupWrapper>>,
  measurements: Arc<DashMap<String, MeasurementWrapper>>,
  scales: Arc<DashMap<String, Scale>>,
  contours: Arc<DashMap<String, ContourWrapper>>,
}
```

**Step 4: Add CRUD methods**

Add to the `#[napi] impl TakeoffStateHandler` block:

```rust
#[napi]
pub fn upsert_contour(&self, contour: ContourInputJs) {
  let input: takeoff_core::contour::ContourInput = contour.into();
  let id = input.id.clone();

  if let Some(existing) = self.contours.get(&id) {
    existing.set_contour(input);
    return;
  }

  let wrapper = ContourWrapper::from_input(input, Arc::new(self.clone()));
  wrapper.calculate_scale();
  self.contours.insert(id, wrapper);
}

#[napi]
pub fn remove_contour(&self, contour_id: String) -> bool {
  self.contours.remove(&contour_id).is_some()
}

#[napi]
pub fn get_contour(&self, contour_id: String) -> Option<ContourWrapper> {
  self.contours.get(&contour_id).map(|entry| entry.value().clone())
}

#[napi]
pub fn get_contours_by_page_id(&self, page_id: String) -> Vec<ContourWrapper> {
  self.contours
    .iter()
    .filter(|entry| entry.value().page_id() == page_id)
    .map(|entry| entry.value().clone())
    .collect()
}

#[napi]
pub fn get_contours_missing_scale(&self) -> Vec<ContourWrapper> {
  self.contours
    .iter()
    .filter(|entry| entry.value().get_scale().is_none())
    .map(|entry| entry.value().clone())
    .collect()
}
```

**Step 5: Add `compute_contours` method**

Add a private method to recompute contours when scales change:

```rust
fn compute_contours(&self, page_id: &str) {
  let contours: Vec<ContourWrapper> = self
    .contours
    .iter()
    .filter(|entry| entry.value().page_id() == page_id)
    .map(|entry| entry.value().clone())
    .collect();
  for contour in contours {
    contour.calculate_scale();
  }
}
```

**Step 6: Wire scale changes to contour recomputation**

In the existing `upsert_scale` method, add `self.compute_contours(&page_id)`:

```rust
// Existing:
pub fn upsert_scale(&self, scale: Scale) -> Option<Scale> {
  let page_id = scale.page_id();
  let res = self.scales.insert(scale.id(), scale);
  self.compute_page(&page_id);
  res
}

// Updated:
pub fn upsert_scale(&self, scale: Scale) -> Option<Scale> {
  let page_id = scale.page_id();
  let res = self.scales.insert(scale.id(), scale);
  self.compute_page(&page_id);
  self.compute_contours(&page_id);
  res
}
```

In the existing `remove_scale` method, add `self.compute_contours(&scale.page_id())`:

```rust
// Updated:
pub fn remove_scale(&self, scale_id: String) -> Option<Scale> {
  let scale = self.scales.remove(&scale_id);
  if let Some((_, scale)) = scale {
    self.compute_page(&scale.page_id());
    self.compute_contours(&scale.page_id());
    return Some(scale);
  }
  None
}
```

**Step 7: Run test**

Run: `cargo test -p takeoff-calculator test_upsert_contour_with_deferred_scale`
Expected: PASS

**Step 8: Run all bindings tests**

Run: `cargo test -p takeoff-calculator`
Expected: PASS

**Step 9: Commit**

```bash
git add packages/bindings/src/state.rs packages/bindings/src/contour.rs
git commit -m "feat: integrate contours into TakeoffStateHandler with deferred mesh"
```

---

### Task 8: Update TypeScript tests

**Files:**
- Modify: `packages/bindings/__test__/contour.test.ts`

**Step 1: Build the native bindings**

Run: `pnpm build`
Expected: Build succeeds

**Step 2: Update test to use new input format with `unit` field**

Rewrite `packages/bindings/__test__/contour.test.ts`:

```typescript
import { describe, expect, test } from "vitest";
import { ContourWrapper, TakeoffStateHandler } from "../index.js";

describe("ContourWrapper", () => {
	test("should create a contour wrapper with unit-based elevation", () => {
		const contour = new ContourWrapper({
			id: "test-contour",
			name: "Test Contour",
			pageId: "test-page",
			lines: [
				{
					elevation: 0,
					unit: "Feet",
					points: [
						{ x: 0, y: 0 },
						{ x: 99.9, y: 0 },
						{ x: 99.9, y: 99.9 },
						{ x: 0, y: 99.9 },
					],
				},
			],
			pointsOfInterest: [],
		});
		expect(contour).toBeDefined();
		// No scale set via constructor — mesh should be None
		expect(contour.getScatterData(10)).toBeNull();
	});

	test("should work with state handler and deferred scale", () => {
		const state = new TakeoffStateHandler();

		state.upsertContour({
			id: "c1",
			pageId: "p1",
			lines: [
				{
					elevation: 0,
					unit: "Feet",
					points: [
						{ x: 0, y: 0 },
						{ x: 99.9, y: 0 },
						{ x: 99.9, y: 99.9 },
						{ x: 0, y: 99.9 },
					],
				},
			],
			pointsOfInterest: [],
		});

		// No scale yet
		let contour = state.getContour("c1");
		expect(contour).toBeDefined();
		expect(contour?.getSurfacePoints()).toBeNull();

		// Add scale
		state.upsertScale({
			type: "Default",
			id: "s1",
			pageId: "p1",
			scale: {
				pixelDistance: 1,
				realDistance: 1,
				unit: "Feet",
			},
		});

		// Now mesh should be available
		contour = state.getContour("c1");
		expect(contour?.getSurfacePoints()).toBeDefined();
		const points = contour?.getSurfacePoints();
		expect(points?.length).toBe(4);
	});

	test("should compute volume with units", () => {
		const state = new TakeoffStateHandler();

		state.upsertScale({
			type: "Default",
			id: "s1",
			pageId: "p1",
			scale: {
				pixelDistance: 1,
				realDistance: 1,
				unit: "Feet",
			},
		});

		state.upsertContour({
			id: "c1",
			pageId: "p1",
			lines: [
				{
					elevation: 0,
					unit: "Feet",
					points: [
						{ x: 0, y: 0 },
						{ x: 99.9, y: 0 },
						{ x: 99.9, y: 99.9 },
						{ x: 0, y: 99.9 },
					],
				},
			],
			pointsOfInterest: [],
		});

		const contour = state.getContour("c1");
		expect(contour).toBeDefined();

		const rawVolume = contour?.rawVolumeAgainst({
			type: "Rectangle",
			points: [
				{ x: 25, y: 25 },
				{ x: 75, y: 75 },
			],
			elevation: 0,
		});
		expect(rawVolume).toBeDefined();
		expect(rawVolume?.cut).toBe(0);
		expect(rawVolume?.fill).toBe(0);

		const unitVolume = contour?.volumeAgainst({
			type: "Rectangle",
			points: [
				{ x: 25, y: 25 },
				{ x: 75, y: 75 },
			],
			elevation: 0,
		});
		expect(unitVolume).toBeDefined();
	});

	test("should get z data for a raised point in a contour", () => {
		const state = new TakeoffStateHandler();

		state.upsertScale({
			type: "Default",
			id: "s1",
			pageId: "p1",
			scale: {
				pixelDistance: 1,
				realDistance: 1,
				unit: "Feet",
			},
		});

		state.upsertContour({
			id: "c1",
			pageId: "p1",
			lines: [
				{
					elevation: 0,
					unit: "Feet",
					points: [
						{ x: 0, y: 0 },
						{ x: 99.9, y: 0 },
						{ x: 99.9, y: 99.9 },
						{ x: 0, y: 99.9 },
					],
				},
			],
			pointsOfInterest: [
				{
					elevation: 100,
					unit: "Feet",
					point: { x: 50, y: 50 },
				},
			],
		});

		const contour = state.getContour("c1");
		const z = contour?.getZAt(50, 50);
		expect(z).toBeDefined();
		expect(z).toBe(100);
	});
});
```

**Step 3: Run TypeScript tests**

Run: `pnpm test`
Expected: PASS

**Step 4: Commit**

```bash
git add packages/bindings/__test__/contour.test.ts
git commit -m "test: update contour tests for unit-based elevation input"
```

---

### Task 9: Run full test suite and verify

**Files:** None (verification only)

**Step 1: Run Rust tests**

Run: `pnpm test:rust`
Expected: All tests pass

**Step 2: Run TypeScript tests**

Run: `pnpm test`
Expected: All tests pass

**Step 3: Run linting and formatting**

Run: `pnpm check`
Expected: No errors

**Step 4: Fix any issues found**

Address clippy warnings, formatting issues, or test failures.

**Step 5: Final commit if any fixes**

```bash
git add -A
git commit -m "chore: fix lint and formatting issues"
```
