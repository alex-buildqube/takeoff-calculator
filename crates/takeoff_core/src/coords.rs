use geo::{Coord, Point as GeoPoint, coord};
use napi_derive::napi;
use serde::{Deserialize, Serialize};

/// Represents a 2D point with floating point coordinates
#[napi(object)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
  pub x: f64,
  pub y: f64,
}

impl Point {
  pub fn new(x: f64, y: f64) -> Self {
    Self { x, y }
  }

  /// Calculate distance between two points
  pub fn distance_to(&self, other: &Point) -> f64 {
    let dx = self.x - other.x;
    let dy = self.y - other.y;
    (dx * dx + dy * dy).sqrt()
  }
}

impl From<Point> for Coord<f64> {
  fn from(p: Point) -> Self {
    // Coord::<f64>::new(p.x, p.y)
    coord! { x: p.x, y: p.y }
  }
}

impl From<Coord<f64>> for Point {
  fn from(c: Coord<f64>) -> Self {
    Point::new(c.x, c.y)
  }
}

impl From<GeoPoint<f64>> for Point {
  fn from(p: GeoPoint<f64>) -> Self {
    Point::new(p.x(), p.y())
  }
}

impl From<Point> for GeoPoint<f64> {
  fn from(p: Point) -> Self {
    GeoPoint::new(p.x, p.y)
  }
}

#[napi(object)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point3D {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

impl Point3D {
  pub fn new(x: f64, y: f64, z: f64) -> Self {
    Self { x, y, z }
  }
}

#[cfg(test)]
mod tests {
  use geo::{Distance, Euclidean};

  use super::*;

  #[test]
  fn test_distance_to() {
    let point1 = Point::new(43.0, 55.0);
    let point2 = Point::new(0.0, 0.0);
    // assert_eq!(point1.distance_to(&point2), 76.0);

    let start: GeoPoint<f64> = (point1).into();
    let end: GeoPoint<f64> = (point2).into();
    assert_eq!(
      Euclidean.distance(&start, &end),
      point1.distance_to(&point2)
    );
    assert_eq!(Euclidean.distance(&start, &end).round(), 70.0);
  }
}
