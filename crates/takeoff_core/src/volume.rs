//! Volumetric cut/fill calculations between a surface mesh and a reference polygon at constant elevation.

use crate::contour::SurfaceMesh;
use crate::coords::Point;
use geo::{Area, Contains, Coord, CoordsIter, LineString, Point as GeoPoint, Polygon, Rect};
use napi_derive::napi;
use serde::{Deserialize, Serialize};

/// Input for creating a reference surface from JS/TS.
#[napi(discriminant = "type")]
#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceSurfaceInput {
  Polygon {
    points: Vec<Point>,
    elevation: f64,
  },
  Rectangle {
    points: (Point, Point),
    elevation: f64,
  },
}

impl ReferenceSurfaceInput {
  pub fn to_polygon(&self) -> Polygon<f64> {
    match self {
      ReferenceSurfaceInput::Polygon { points, .. } => {
        let points: Vec<Coord<f64>> = points.iter().map(|p| (*p).into()).collect();
        Polygon::new(LineString::from(points), vec![])
      }
      ReferenceSurfaceInput::Rectangle { points, .. } => {
        let start: Coord<f64> = points.0.into();
        let end: Coord<f64> = points.1.into();
        let rect = Rect::new(start, end);
        rect.to_polygon()
      }
    }
  }
}

/// A reference surface: a 2D polygon extruded at a constant elevation (e.g. foundation footprint).
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceSurface {
  pub polygon: Polygon<f64>,
  pub elevation: f64,
}

impl ReferenceSurface {
  /// Create a reference surface from an exterior ring and constant elevation.
  pub fn new(exterior: Vec<Point>, elevation: f64) -> Self {
    let coords: Vec<Coord<f64>> = exterior
      .iter()
      .map(|p| Into::<Coord<f64>>::into(*p))
      .collect();
    let polygon = Polygon::new(LineString::from(coords), vec![]);
    Self { polygon, elevation }
  }

  /// Create from a geo polygon and elevation.
  pub fn from_polygon(polygon: Polygon<f64>, elevation: f64) -> Self {
    Self { polygon, elevation }
  }

  /// Polygon area (for default cell size calculation).
  fn area(&self) -> f64 {
    self.polygon.unsigned_area()
  }

  /// Bounding box (min_x, min_y, max_x, max_y).
  fn bounding_box(&self) -> Option<(f64, f64, f64, f64)> {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut any = false;
    for c in self.polygon.exterior_coords_iter() {
      any = true;
      min_x = min_x.min(c.x);
      min_y = min_y.min(c.y);
      max_x = max_x.max(c.x);
      max_y = max_y.max(c.y);
    }
    if any {
      Some((min_x, min_y, max_x, max_y))
    } else {
      None
    }
  }
}

impl From<ReferenceSurfaceInput> for ReferenceSurface {
  fn from(input: ReferenceSurfaceInput) -> Self {
    // ReferenceSurface::new(input.exterior, input.elevation)
    match input {
      ReferenceSurfaceInput::Polygon { elevation, .. } => {
        ReferenceSurface::from_polygon(input.to_polygon(), elevation)
      }
      ReferenceSurfaceInput::Rectangle { elevation, .. } => {
        ReferenceSurface::from_polygon(input.to_polygon(), elevation)
      }
    }
  }
}

/// Result of a volumetric cut/fill calculation.
#[napi(object)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VolumetricResult {
  /// Volume to remove (terrain above reference).
  pub cut: f64,
  /// Volume to add (terrain below reference).
  pub fill: f64,
  /// Area where terrain data was unavailable (z_at returned None).
  pub uncovered_area: f64,
}

impl SurfaceMesh {
  /// Compute cut/fill volume against a reference surface using grid sampling.
  /// Uses `cell_size` if provided; otherwise defaults to `sqrt(polygon_area / 1000)`.
  pub fn volume_against(
    &self,
    reference: &ReferenceSurface,
    cell_size: Option<f64>,
  ) -> VolumetricResult {
    let bbox = match reference.bounding_box() {
      Some(b) => b,
      None => {
        return VolumetricResult {
          cut: 0.0,
          fill: 0.0,
          uncovered_area: 0.0,
        };
      }
    };

    let area = reference.area();
    if area <= 0.0 {
      return VolumetricResult {
        cut: 0.0,
        fill: 0.0,
        uncovered_area: 0.0,
      };
    }

    let cell_size = cell_size.unwrap_or_else(|| (area / 1000.0).sqrt());
    let cell_area = cell_size * cell_size;

    let (min_x, min_y, max_x, max_y) = bbox;
    let ref_elev = reference.elevation;

    let mut cut = 0.0;
    let mut fill = 0.0;
    let mut uncovered_area = 0.0;

    let mut x = min_x + cell_size / 2.0;
    while x < max_x {
      let mut y = min_y + cell_size / 2.0;
      while y < max_y {
        let point = GeoPoint::new(x, y);
        if reference.polygon.contains(&point) {
          match self.z_at(x, y) {
            Some(terrain_z) => {
              let delta = terrain_z - ref_elev;
              if delta > 0.0 {
                cut += cell_area * delta;
              } else if delta < 0.0 {
                fill += cell_area * (-delta);
              }
            }
            None => uncovered_area += cell_area,
          }
        }
        y += cell_size;
      }
      x += cell_size;
    }

    VolumetricResult {
      cut,
      fill,
      uncovered_area,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::coords::Point3D;

  #[test]
  fn test_volume_flat_terrain_above_reference() {
    let mesh = SurfaceMesh {
      vertices: vec![
        Point3D::new(0.0, 0.0, 10.0),
        Point3D::new(10.0, 0.0, 10.0),
        Point3D::new(10.0, 10.0, 10.0),
        Point3D::new(0.0, 10.0, 10.0),
      ],
      triangles: vec![[0, 1, 2], [0, 2, 3]],
    };
    let reference = ReferenceSurface::new(
      vec![
        Point::new(2.0, 2.0),
        Point::new(8.0, 2.0),
        Point::new(8.0, 8.0),
        Point::new(2.0, 8.0),
      ],
      5.0,
    );
    let result = mesh.volume_against(&reference, Some(1.0));
    assert!(result.cut > 0.0, "expected cut > 0");
    assert!(result.fill < 1e-9, "expected fill ~ 0");
    assert!(result.uncovered_area < 1e-9, "expected no uncovered area");
    let expected_cut_approx = 6.0 * 6.0 * 5.0;

    assert!(
      (result.cut - expected_cut_approx).abs() < expected_cut_approx * 0.2,
      "cut should be ~{} (area 36 * height 5), got {}",
      expected_cut_approx,
      result.cut
    );
  }

  #[test]
  fn test_volume_flat_terrain_below_reference() {
    let mesh = SurfaceMesh {
      vertices: vec![
        Point3D::new(0.0, 0.0, 2.0),
        Point3D::new(10.0, 0.0, 2.0),
        Point3D::new(10.0, 10.0, 2.0),
        Point3D::new(0.0, 10.0, 2.0),
      ],
      triangles: vec![[0, 1, 2], [0, 2, 3]],
    };
    let reference = ReferenceSurface::new(
      vec![
        Point::new(2.0, 2.0),
        Point::new(8.0, 2.0),
        Point::new(8.0, 8.0),
        Point::new(2.0, 8.0),
      ],
      5.0,
    );
    let result = mesh.volume_against(&reference, Some(1.0));
    assert!(result.fill > 0.0, "expected fill > 0");
    assert!(result.cut < 1e-9, "expected cut ~ 0");
    let expected_fill_approx = 6.0 * 6.0 * 3.0;
    assert!(
      (result.fill - expected_fill_approx).abs() < expected_fill_approx * 0.2,
      "fill should be ~{}, got {}",
      expected_fill_approx,
      result.fill
    );
  }

  #[test]
  fn test_volume_polygon_outside_mesh() {
    let mesh = SurfaceMesh {
      vertices: vec![
        Point3D::new(0.0, 0.0, 10.0),
        Point3D::new(10.0, 0.0, 10.0),
        Point3D::new(5.0, 10.0, 10.0),
      ],
      triangles: vec![[0, 1, 2]],
    };
    let reference = ReferenceSurface::new(
      vec![
        Point::new(100.0, 100.0),
        Point::new(110.0, 100.0),
        Point::new(110.0, 110.0),
        Point::new(100.0, 110.0),
      ],
      5.0,
    );
    let result = mesh.volume_against(&reference, Some(1.0));
    assert!(result.uncovered_area > 0.0, "expected uncovered area");
    assert!(result.cut < 1e-9);
    assert!(result.fill < 1e-9);
  }

  #[test]
  fn test_volume_degenerate_polygon() {
    let mesh = SurfaceMesh {
      vertices: vec![
        Point3D::new(0.0, 0.0, 10.0),
        Point3D::new(10.0, 0.0, 10.0),
        Point3D::new(5.0, 10.0, 10.0),
      ],
      triangles: vec![[0, 1, 2]],
    };
    let reference = ReferenceSurface::new(
      vec![
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        Point::new(2.0, 0.0),
      ],
      5.0,
    );
    let result = mesh.volume_against(&reference, Some(1.0));
    assert_eq!(result.cut, 0.0);
    assert_eq!(result.fill, 0.0);
  }
}
