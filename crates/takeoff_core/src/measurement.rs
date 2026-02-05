use crate::coords::Point;
use geo::{Area, Coord, CoordsIter, Geometry, LineString, Polygon as GeoPolygon, Rect};
use napi_derive::napi;
use serde::{Deserialize, Serialize};

#[napi(discriminant = "type")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Measurement {
  Count {
    id: String,
    page_id: String,
    group_id: String,
    points: (Point,),
  },
  Polygon {
    id: String,
    page_id: String,
    group_id: String,
    points: Vec<Point>,
  },
  Polyline {
    id: String,
    page_id: String,
    group_id: String,
    points: Vec<Point>,
  },
  Rectangle {
    id: String,
    page_id: String,
    group_id: String,
    points: (Point, Point),
  },
}

impl Measurement {
  /// Get the id of the measurement
  pub fn id(&self) -> &str {
    match self {
      Measurement::Count { id, .. } => id,
      Measurement::Polygon { id, .. } => id,
      Measurement::Polyline { id, .. } => id,
      Measurement::Rectangle { id, .. } => id,
    }
  }
  /// Get the page id of the measurement
  pub fn page_id(&self) -> &str {
    match self {
      Measurement::Count { page_id, .. } => page_id,
      Measurement::Polygon { page_id, .. } => page_id,
      Measurement::Polyline { page_id, .. } => page_id,
      Measurement::Rectangle { page_id, .. } => page_id,
    }
  }
  /// Get the group id of the measurement
  pub fn group_id(&self) -> &str {
    match self {
      Measurement::Count { group_id, .. } => group_id,
      Measurement::Polygon { group_id, .. } => group_id,
      Measurement::Polyline { group_id, .. } => group_id,
      Measurement::Rectangle { group_id, .. } => group_id,
    }
  }

  /// Convert the measurement to a polygon
  pub fn to_polygon(&self) -> Option<GeoPolygon<f64>> {
    match self {
      Measurement::Polygon { points, .. } => {
        let points: Vec<Coord<f64>> = points.iter().map(|p| (*p).into()).collect();
        Some(GeoPolygon::new(LineString::from(points), vec![]))
      }
      Measurement::Rectangle { points, .. } => {
        let start: Coord<f64> = points.0.into();
        let end: Coord<f64> = points.1.into();
        let rect = Rect::new(start, end);
        Some(rect.to_polygon())
      }
      _ => None,
    }
  }

  pub fn to_line_string(&self) -> Option<LineString<f64>> {
    match self {
      Measurement::Polyline { points, .. } => Some(LineString::new(
        points.iter().map(|p| (*p).into()).collect(),
      )),
      Measurement::Rectangle { .. } => Some(self.to_polygon().unwrap().exterior().clone()),
      Measurement::Polygon { .. } => Some(self.to_polygon().unwrap().exterior().clone()),
      Measurement::Count { .. } => None,
    }
  }

  pub fn to_point(&self) -> Point {
    match self {
      Measurement::Count { points, .. } => points.0,
      Measurement::Polygon { points, .. } => *points.first().unwrap(),
      Measurement::Polyline { points, .. } => *points.first().unwrap(),
      Measurement::Rectangle { points, .. } => points.0,
    }
  }

  pub fn to_geometry(&self) -> Geometry<f64> {
    match self {
      Measurement::Polygon { .. } => Geometry::Polygon(self.to_polygon().unwrap()),
      Measurement::Rectangle { .. } => Geometry::Polygon(self.to_polygon().unwrap()),
      Measurement::Polyline { .. } => Geometry::LineString(self.to_line_string().unwrap()),
      Measurement::Count { .. } => Geometry::Point(self.to_point().into()),
    }
  }
  /// Calculate the area of the polygon
  pub fn pixel_area(&self) -> f64 {
    let polygon = self.to_polygon();
    if let Some(polygon) = polygon {
      polygon.unsigned_area()
    } else {
      0.0
    }
  }

  /// Calculate the perimeter of the rectangle
  pub fn pixel_perimeter(&self) -> f64 {
    match self {
      Measurement::Polygon { points, .. } => {
        let mut perimeter = 0.0;
        for i in 0..points.len() {
          let j = (i + 1) % points.len();
          perimeter += points[i].distance_to(&points[j]);
        }
        perimeter
      }
      Measurement::Rectangle { .. } => {
        let polygon = self.to_polygon().unwrap();
        let coords: Vec<Point> = polygon.exterior_coords_iter().map(Point::from).collect();
        let mut perimeter = 0.0;
        for i in 0..coords.len() {
          let j = (i + 1) % coords.len();
          perimeter += coords[i].distance_to(&coords[j]);
        }
        perimeter
      }
      Measurement::Polyline { points, .. } => {
        let mut perimeter = 0.0;
        for i in 0..points.len() {
          let j = (i + 1) % points.len();
          perimeter += points[i].distance_to(&points[j]);
        }
        perimeter
      }
      Measurement::Count { .. } => 0.0,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_pixel_area() {
    let measurement = Measurement::Rectangle {
      id: "1".to_string(),
      page_id: "1".to_string(),
      group_id: "1".to_string(),
      points: (Point::new(0.0, 0.0), Point::new(100.0, 50.0)),
    };
    assert!(measurement.pixel_area() == 5000.0);
  }
  #[test]
  fn test_pixel_perimeter() {
    let measurement = Measurement::Rectangle {
      id: "1".to_string(),
      page_id: "1".to_string(),
      group_id: "1".to_string(),
      points: (Point::new(0.0, 0.0), Point::new(100.0, 50.0)),
    };
    assert!(measurement.pixel_perimeter() == 300.0);
  }
}
