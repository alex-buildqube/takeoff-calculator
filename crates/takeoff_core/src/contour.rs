use crate::{
  TakeoffError,
  coords::{Point, Point3D},
  error::TakeoffResult,
};
use delaunator::triangulate;
use geo::{BoundingRect, Geometry, GeometryCollection, LineString, Point as GeoPoint};
use napi_derive::napi;
use serde::{Deserialize, Serialize};

#[napi(object)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContourLineInput {
  /// The elevation of the contour line (in pixels)
  pub elevation: f64,
  pub points: Vec<Point>,
}

impl ContourLineInput {
  pub fn to_geometry(&self) -> LineString<f64> {
    LineString::from(self.points.clone())
  }
}

impl From<ContourLineInput> for Vec<Point3D> {
  fn from(input: ContourLineInput) -> Self {
    input
      .points
      .into_iter()
      .map(|p| Point3D {
        x: p.x,
        y: p.y,
        z: input.elevation,
      })
      .collect()
  }
}

#[napi(object)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContourPointOfInterestInput {
  /// The elevation of the point of interest (in pixels)
  pub elevation: f64,
  pub point: Point,
}

impl From<ContourPointOfInterestInput> for Point3D {
  fn from(input: ContourPointOfInterestInput) -> Self {
    Point3D {
      x: input.point.x,
      y: input.point.y,
      z: input.elevation,
    }
  }
}

#[napi(object)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContourInput {
  pub id: String,
  pub name: Option<String>,
  pub page_id: String,

  /// The lines that make up the contour map
  pub lines: Vec<ContourLineInput>,
  /// The points of interest that are used to create the contour map
  pub points_of_interest: Vec<ContourPointOfInterestInput>,
}

/// A triangulated 3D surface mesh suitable for volumetric calculations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurfaceMesh {
  pub vertices: Vec<Point3D>,
  pub triangles: Vec<[u32; 3]>,
}

impl SurfaceMesh {
  const DEDUP_EPSILON: f64 = 1e-9;

  const Z_AT_VERTEX_EPSILON: f64 = 1e-12;

  /// Returns the interpolated z value at (x, y) if the point lies within the mesh boundary.
  /// Uses barycentric interpolation over the containing triangle.
  /// Returns `None` if the point is outside the mesh.
  pub fn z_at(&self, x: f64, y: f64) -> Option<f64> {
    for v in &self.vertices {
      if (v.x - x).abs() < Self::Z_AT_VERTEX_EPSILON && (v.y - y).abs() < Self::Z_AT_VERTEX_EPSILON
      {
        return Some(v.z);
      }
    }
    for tri in &self.triangles {
      let a = &self.vertices[tri[0] as usize];
      let b = &self.vertices[tri[1] as usize];
      let c = &self.vertices[tri[2] as usize];

      let v0x = b.x - a.x;
      let v0y = b.y - a.y;
      let v1x = c.x - a.x;
      let v1y = c.y - a.y;
      let v2x = x - a.x;
      let v2y = y - a.y;

      let dot00 = v0x * v0x + v0y * v0y;
      let dot01 = v0x * v1x + v0y * v1y;
      let dot02 = v0x * v2x + v0y * v2y;
      let dot11 = v1x * v1x + v1y * v1y;
      let dot12 = v1x * v2x + v1y * v2y;

      let denom = dot00 * dot11 - dot01 * dot01;
      if denom.abs() < 1e-12 {
        continue;
      }
      let inv_denom = 1.0 / denom;

      let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
      let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;
      let w = 1.0 - u - v;

      if u >= -1e-12 && v >= -1e-12 && (u + v) <= 1.0 + 1e-12 {
        return Some(w * a.z + u * b.z + v * c.z);
      }
    }
    None
  }

  /// Deduplicate points by (x, y) within tolerance. Keeps first z when duplicates occur.
  fn deduplicate_points(points: &[Point3D]) -> Vec<Point3D> {
    let mut seen: Vec<Point3D> = Vec::new();
    for p in points {
      let is_dup = seen.iter().any(|s| {
        (s.x - p.x).abs() < Self::DEDUP_EPSILON && (s.y - p.y).abs() < Self::DEDUP_EPSILON
      });
      if !is_dup {
        seen.push(*p);
      }
    }
    seen
  }
}

impl TryFrom<ContourInput> for SurfaceMesh {
  type Error = TakeoffError;

  fn try_from(input: ContourInput) -> Result<Self, Self::Error> {
    let points = input.get_points();

    let vertices = Self::deduplicate_points(&points);

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

    Ok(Self {
      vertices,
      triangles,
    })
  }
}

impl ContourInput {
  /// Convert contour input to a triangulated 3D surface mesh.
  ///
  /// # Errors
  ///
  /// Returns [`TakeoffError::SurfaceMeshTooFewPoints`] if there are fewer than 3 points.
  /// Returns [`TakeoffError::SurfaceMeshCollinearPoints`] if all points are collinear.
  pub fn to_surface_mesh(&self) -> TakeoffResult<SurfaceMesh> {
    SurfaceMesh::try_from(self.clone())
  }

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

  fn get_geometry_collection(&self) -> GeometryCollection {
    let line_geometries = self
      .lines
      .iter()
      .map(|line| Geometry::LineString(line.to_geometry()));
    let point_geometries = self
      .points_of_interest
      .iter()
      .map(|point| Geometry::Point(GeoPoint::new(point.point.x, point.point.y)));

    let geometries = line_geometries.chain(point_geometries).collect();

    GeometryCollection::new_from(geometries)
  }

  /// Get contour bounding box
  pub fn bounding_box(&self) -> Option<((f64, f64), (f64, f64))> {
    let geometry_collection = self.get_geometry_collection();
    let bounding_box = geometry_collection.bounding_rect();
    if let Some(bounding_box) = bounding_box {
      let min = bounding_box.min();
      let max = bounding_box.max();
      let min_x = min.x;
      let min_y = min.y;
      let max_x = max.x;
      let max_y = max.y;
      return Some(((min_x, min_y), (max_x, max_y)));
    }
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_surface_mesh_success() {
    let input = ContourInput {
      id: "1".to_string(),
      name: Some("test".to_string()),
      page_id: "1".to_string(),
      lines: vec![ContourLineInput {
        elevation: 10.0,
        points: vec![
          Point::new(0.0, 0.0),
          Point::new(10.0, 0.0),
          Point::new(10.0, 10.0),
          Point::new(0.0, 10.0),
        ],
      }],
      points_of_interest: vec![ContourPointOfInterestInput {
        elevation: 5.0,
        point: Point::new(5.0, 5.0),
      }],
    };
    let mesh = input.to_surface_mesh().unwrap();
    assert_eq!(mesh.vertices.len(), 5);
    assert!(!mesh.triangles.is_empty());

    assert!(
      mesh
        .triangles
        .iter()
        .all(|t| t[0] < 5 && t[1] < 5 && t[2] < 5)
    );
  }

  #[test]
  fn test_surface_mesh_too_few_points() {
    let input = ContourInput {
      id: "1".to_string(),
      name: None,
      page_id: "1".to_string(),
      lines: vec![ContourLineInput {
        elevation: 10.0,
        points: vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0)],
      }],
      points_of_interest: vec![],
    };
    let err = input.to_surface_mesh().unwrap_err();
    assert!(matches!(
      err,
      TakeoffError::SurfaceMeshTooFewPoints { count: 2 }
    ));
  }

  #[test]
  fn test_surface_mesh_collinear() {
    let input = ContourInput {
      id: "1".to_string(),
      name: None,
      page_id: "1".to_string(),
      lines: vec![ContourLineInput {
        elevation: 10.0,
        points: vec![
          Point::new(0.0, 0.0),
          Point::new(5.0, 0.0),
          Point::new(10.0, 0.0),
        ],
      }],
      points_of_interest: vec![],
    };
    let err = input.to_surface_mesh().unwrap_err();
    assert!(matches!(err, TakeoffError::SurfaceMeshCollinearPoints));
  }

  #[test]
  fn test_surface_mesh_deduplication() {
    let input = ContourInput {
      id: "1".to_string(),
      name: None,
      page_id: "1".to_string(),
      lines: vec![
        ContourLineInput {
          elevation: 10.0,
          points: vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0)],
        },
        ContourLineInput {
          elevation: 20.0,
          points: vec![Point::new(0.0, 0.0), Point::new(0.0, 10.0)],
        },
      ],
      points_of_interest: vec![ContourPointOfInterestInput {
        elevation: 5.0,
        point: Point::new(5.0, 5.0),
      }],
    };
    let mesh = input.to_surface_mesh().unwrap();
    assert_eq!(mesh.vertices.len(), 4);
  }

  #[test]
  fn test_z_at_at_vertex() {
    let input = ContourInput {
      id: "1".to_string(),
      name: None,
      page_id: "1".to_string(),
      lines: vec![ContourLineInput {
        elevation: 10.0,
        points: vec![
          Point::new(0.0, 0.0),
          Point::new(10.0, 0.0),
          Point::new(10.0, 10.0),
          Point::new(0.0, 10.0),
        ],
      }],
      points_of_interest: vec![ContourPointOfInterestInput {
        elevation: 5.0,
        point: Point::new(5.0, 5.0),
      }],
    };
    let mesh = input.to_surface_mesh().unwrap();
    assert_eq!(mesh.z_at(0.0, 0.0), Some(10.0));
    assert_eq!(mesh.z_at(5.0, 5.0), Some(5.0));
    assert_eq!(mesh.z_at(10.0, 10.0), Some(10.0));
  }

  #[test]
  fn test_z_at_interpolation() {
    let mesh = SurfaceMesh {
      vertices: vec![
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(10.0, 0.0, 10.0),
        Point3D::new(5.0, 10.0, 5.0),
      ],
      triangles: vec![[0, 1, 2]],
    };
    let z = mesh.z_at(5.0, 5.0).unwrap();
    assert!((z - 5.0).abs() < 1e-6, "expected ~5.0, got {}", z);
    let z_center = mesh.z_at(5.0, 10.0 / 3.0).unwrap();
    let expected_center = (0.0 + 10.0 + 5.0) / 3.0;
    assert!(
      (z_center - expected_center).abs() < 1e-6,
      "expected ~{} at centroid, got {}",
      expected_center,
      z_center
    );
  }

  #[test]
  fn test_z_at_outside_mesh() {
    let mesh = SurfaceMesh {
      vertices: vec![
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(10.0, 0.0, 0.0),
        Point3D::new(5.0, 10.0, 0.0),
      ],
      triangles: vec![[0, 1, 2]],
    };
    assert_eq!(mesh.z_at(-1.0, -1.0), None);
    assert_eq!(mesh.z_at(100.0, 100.0), None);
    assert_eq!(mesh.z_at(5.0, -1.0), None);
  }

  #[test]
  fn test_z_at_on_edge() {
    let mesh = SurfaceMesh {
      vertices: vec![
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(10.0, 0.0, 10.0),
        Point3D::new(5.0, 10.0, 5.0),
      ],
      triangles: vec![[0, 1, 2]],
    };
    let z = mesh.z_at(5.0, 0.0).unwrap();
    assert!(
      (z - 5.0).abs() < 1e-6,
      "midpoint of edge (0,0)-(10,0) should be 5.0, got {}",
      z
    );
  }
}
