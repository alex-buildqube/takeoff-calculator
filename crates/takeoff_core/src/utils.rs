use crate::coords::Point;
use crate::measurement::Measurement;
use geo::LineString;
use geo::Simplify;

use napi_derive::napi;

/// Simplify a polyline using the Ramer-Douglas-Peucker algorithm
#[napi]
pub fn simplify_polyline(points: Vec<Point>, tolerance: f64) -> Vec<Point> {
  let line_string = LineString::new(points.iter().map(|p| (*p).into()).collect());

  let simplified = line_string.simplify(tolerance);
  simplified.into_iter().map(Point::from).collect()
}

/// Get the centroid of a measurement
#[napi]
pub fn get_centroid(measurement: Measurement) -> Option<Point> {
  measurement.get_centroid()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_simplify_polyline() {
    let points = vec![
      Point::new(0.0, 0.0),
      Point::new(1.0, 0.0),
      Point::new(1.0, 1.0),
      Point::new(2.0, 2.0),
    ];
    let simplified = simplify_polyline(points, 0.5);
    assert_eq!(
      simplified,
      vec![
        Point::new(0.0, 0.0),
        Point { x: 1.0, y: 0.0 },
        Point::new(2.0, 2.0)
      ]
    );
  }

  #[test]
  fn test_get_centroid() {
    let measurement = Measurement::Rectangle {
      id: "1".to_string(),
      page_id: "1".to_string(),
      group_id: "1".to_string(),
      points: (Point::new(0.0, 0.0), Point::new(1.0, 1.0)),
    };

    let centroid = get_centroid(measurement);
    assert_eq!(centroid, Some(Point::new(0.5, 0.5)));
  }
}
