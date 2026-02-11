use crate::coords::{DistanceTrait, Point};
use crate::error::{TakeoffError, TakeoffResult};
use geo::{Area, Centroid, Coord, CoordsIter, Geometry, LineString, Polygon as GeoPolygon, Rect};
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
  /// Validate that the measurement has valid geometry.
  ///
  /// # Errors
  ///
  /// Returns [`TakeoffError::EmptyGeometry`] if:
  /// - Polygon has fewer than 3 points
  /// - Polyline has fewer than 2 points
  /// - Rectangle has invalid or identical corner points
  pub fn validate(&self) -> TakeoffResult<()> {
    match self {
      Measurement::Polygon { points, .. } => {
        if points.len() < 3 {
          return Err(TakeoffError::empty_geometry(format!(
            "polygon must have at least 3 points, got {}",
            points.len()
          )));
        }
        Ok(())
      }
      Measurement::Polyline { points, .. } => {
        if points.len() < 2 {
          return Err(TakeoffError::empty_geometry(format!(
            "polyline must have at least 2 points, got {}",
            points.len()
          )));
        }
        Ok(())
      }
      Measurement::Rectangle { points, .. } => {
        let (p1, p2) = points;
        if (p1.x - p2.x).abs() < f64::EPSILON && (p1.y - p2.y).abs() < f64::EPSILON {
          return Err(TakeoffError::empty_geometry(
            "rectangle corners must be distinct points",
          ));
        }
        Ok(())
      }
      Measurement::Count { .. } => Ok(()), // Count always has valid geometry (single point)
    }
  }

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

  /// Convert the measurement to a polygon.
  ///
  /// # Errors
  ///
  /// Returns [`TakeoffError::EmptyGeometry`] if:
  /// - The geometry is invalid (e.g., polygon with < 3 points)
  /// - The measurement type cannot be converted to a polygon
  pub fn to_polygon(&self) -> TakeoffResult<GeoPolygon<f64>> {
    self.validate()?;
    match self {
      Measurement::Polygon { points, .. } => {
        let points: Vec<Coord<f64>> = points.iter().map(|p| (*p).into()).collect();
        Ok(GeoPolygon::new(LineString::from(points), vec![]))
      }
      Measurement::Rectangle { points, .. } => {
        let start: Coord<f64> = points.0.into();
        let end: Coord<f64> = points.1.into();
        let rect = Rect::new(start, end);
        Ok(rect.to_polygon())
      }
      _ => Err(TakeoffError::empty_geometry(
        "measurement cannot be converted to polygon",
      )),
    }
  }

  /// Convert the measurement to a line string.
  ///
  /// # Errors
  ///
  /// Returns [`TakeoffError::EmptyGeometry`] if:
  /// - The geometry is invalid (e.g., polyline with < 2 points)
  /// - The measurement type cannot be converted to a line string
  pub fn to_line_string(&self) -> TakeoffResult<LineString<f64>> {
    self.validate()?;
    match self {
      Measurement::Polyline { points, .. } => Ok(LineString::new(
        points.iter().map(|p| (*p).into()).collect(),
      )),
      Measurement::Rectangle { .. } => Ok(self.to_polygon()?.exterior().clone()),
      Measurement::Polygon { .. } => Ok(self.to_polygon()?.exterior().clone()),
      Measurement::Count { .. } => Err(TakeoffError::empty_geometry(
        "count measurement cannot be converted to line string",
      )),
    }
  }

  /// Converts the measurement to a single point.
  ///
  /// For polygons and polylines, returns the first point.
  /// For counts and rectangles, returns the single point or first corner.
  ///
  /// # Errors
  ///
  /// Returns [`TakeoffError::EmptyGeometry`] if:
  /// - The polygon has no points
  /// - The polyline has no points
  pub fn to_point(&self) -> TakeoffResult<Point> {
    match self {
      Measurement::Count { points, .. } => Ok(points.0),
      Measurement::Polygon { points, .. } => {
        if points.is_empty() {
          Err(TakeoffError::empty_geometry("polygon has no points"))
        } else {
          // Safe: we've already checked that points is not empty
          Ok(
            *points
              .first()
              .expect("BUG: points.is_empty() was checked above"),
          )
        }
      }
      Measurement::Polyline { points, .. } => {
        if points.is_empty() {
          Err(TakeoffError::empty_geometry("polyline has no points"))
        } else {
          // Safe: we've already checked that points is not empty
          Ok(
            *points
              .first()
              .expect("BUG: points.is_empty() was checked above"),
          )
        }
      }
      Measurement::Rectangle { points, .. } => Ok(points.0),
    }
  }

  /// Convert the measurement to a geometry object.
  ///
  /// # Errors
  ///
  /// Returns [`TakeoffError::EmptyGeometry`] if the measurement geometry is invalid.
  pub fn to_geometry(&self) -> TakeoffResult<Geometry<f64>> {
    self.validate()?;
    match self {
      Measurement::Polygon { .. } => Ok(Geometry::Polygon(self.to_polygon()?)),
      Measurement::Rectangle { .. } => Ok(Geometry::Polygon(self.to_polygon()?)),
      Measurement::Polyline { .. } => Ok(Geometry::LineString(self.to_line_string()?)),
      Measurement::Count { .. } => Ok(Geometry::Point(self.to_point()?.into())),
    }
  }

  /// Get the centroid (geometric center) of the measurement.
  ///
  /// # Errors
  ///
  /// Returns [`TakeoffError::EmptyGeometry`] if the centroid cannot be computed
  /// (e.g., for empty geometry).
  pub fn get_centroid(&self) -> TakeoffResult<Point> {
    let geometry = self.to_geometry()?;
    let centroid = geometry.centroid();
    centroid
      .map(Point::from)
      .ok_or_else(|| TakeoffError::empty_geometry("cannot compute centroid for empty geometry"))
  }

  /// Calculate the area of the polygon
  ///
  /// Returns an error if the geometry is invalid.
  pub fn pixel_area(&self) -> TakeoffResult<f64> {
    let polygon = self.to_polygon()?;
    Ok(polygon.unsigned_area())
  }

  /// Calculate the perimeter/length of the measurement
  ///
  /// Returns an error if the geometry is invalid.
  pub fn pixel_perimeter(&self) -> TakeoffResult<f64> {
    self.validate()?;
    match self {
      Measurement::Polygon { points, .. } => {
        let mut perimeter = 0.0;
        for i in 0..points.len() {
          let j = (i + 1) % points.len();
          perimeter += points[i].distance_to(&points[j]);
        }
        Ok(perimeter)
      }
      Measurement::Rectangle { .. } => {
        let polygon = self.to_polygon()?;
        let coords: Vec<Point> = polygon.exterior_coords_iter().map(Point::from).collect();
        let mut perimeter = 0.0;
        for i in 0..coords.len() {
          let j = (i + 1) % coords.len();
          perimeter += coords[i].distance_to(&coords[j]);
        }
        Ok(perimeter)
      }
      Measurement::Polyline { points, .. } => {
        let mut perimeter = 0.0;
        // NOTE: This is the correct way to calculate the perimeter of a polyline but we are using the length
        // for i in 0..points.len() {
        //   let j = (i + 1) % points.len();
        //   perimeter += points[i].distance_to(&points[j]);
        // }
        for i in 0..points.len() - 1 {
          perimeter += points[i].distance_to(&points[i + 1]);
        }
        Ok(perimeter)
      }
      Measurement::Count { .. } => Ok(0.0),
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
    assert!(measurement.pixel_area().unwrap() == 5000.0);
  }
  #[test]
  fn test_pixel_perimeter() {
    let measurement = Measurement::Rectangle {
      id: "1".to_string(),
      page_id: "1".to_string(),
      group_id: "1".to_string(),
      points: (Point::new(0.0, 0.0), Point::new(100.0, 50.0)),
    };
    assert!(measurement.pixel_perimeter().unwrap() == 300.0);
  }

  #[test]
  fn test_pixel_perimeter_polyline() {
    let measurement = Measurement::Polyline {
      id: "1".to_string(),
      page_id: "1".to_string(),
      group_id: "1".to_string(),
      points: vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0)],
    };
    assert!(measurement.pixel_perimeter().unwrap() == 1.0);
  }

  #[test]
  fn test_empty_polygon_error() {
    let measurement = Measurement::Polygon {
      id: "1".to_string(),
      page_id: "1".to_string(),
      group_id: "1".to_string(),
      points: vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0)], // Only 2 points
    };
    assert!(matches!(
      measurement.validate(),
      Err(crate::error::TakeoffError::EmptyGeometry { .. })
    ));
    assert!(matches!(
      measurement.pixel_area(),
      Err(crate::error::TakeoffError::EmptyGeometry { .. })
    ));
  }

  #[test]
  fn test_empty_polyline_error() {
    let measurement = Measurement::Polyline {
      id: "1".to_string(),
      page_id: "1".to_string(),
      group_id: "1".to_string(),
      points: vec![Point::new(0.0, 0.0)], // Only 1 point
    };
    assert!(matches!(
      measurement.validate(),
      Err(crate::error::TakeoffError::EmptyGeometry { .. })
    ));
  }

  #[test]
  fn test_invalid_rectangle_error() {
    let measurement = Measurement::Rectangle {
      id: "1".to_string(),
      page_id: "1".to_string(),
      group_id: "1".to_string(),
      points: (Point::new(0.0, 0.0), Point::new(0.0, 0.0)), // Same point
    };
    assert!(matches!(
      measurement.validate(),
      Err(crate::error::TakeoffError::EmptyGeometry { .. })
    ));
  }
}
