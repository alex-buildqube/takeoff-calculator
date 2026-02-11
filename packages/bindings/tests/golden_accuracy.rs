//! Golden / baseline accuracy tests (RFC-007).
//!
//! Compares core conversion outputs to versioned baseline values with a documented
//! tolerance. See `../ACCURACY.md` for policy and `../test_data/README.md` for schema.

use serde::Deserialize;
use takeoff_calculator::measurement::MeasurementWrapper;
use takeoff_core::coords::Point;
use takeoff_core::measurement::Measurement;
use takeoff_core::scale::{Scale, ScaleDefinition};
use takeoff_core::unit::Unit;

/// Relative tolerance for area and length (0.01%).
const RELATIVE_TOLERANCE: f64 = 0.0001;
/// Absolute epsilon for small values; no exact float equality.
const ABSOLUTE_EPSILON: f64 = 1e-10;
/// Below this magnitude we use absolute comparison.
const MIN_MAGNITUDE: f64 = 1e-9;

#[derive(Debug, Deserialize)]
struct PointDto {
  x: f64,
  y: f64,
}

#[derive(Debug, Deserialize)]
struct ScaleDto {
  pixel_distance: f64,
  real_distance: f64,
  unit: String,
}

#[derive(Debug, Deserialize)]
struct ExpectedDto {
  length: Option<f64>,
  area: Option<f64>,
  count: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct BaselineEntry {
  id: String,
  kind: String,
  points: Vec<PointDto>,
  scale: ScaleDto,
  output_unit: String,
  expected: ExpectedDto,
}

fn parse_unit(s: &str) -> Unit {
  Unit::from_str(s).expect("baseline unit must be valid")
}

fn assert_within_tolerance(actual: f64, expected: f64, kind: &str, id: &str) {
  let diff = (actual - expected).abs();
  let ok = if expected.abs() < MIN_MAGNITUDE {
    diff <= ABSOLUTE_EPSILON
  } else {
    let rel = diff / expected.abs().max(MIN_MAGNITUDE);
    diff <= ABSOLUTE_EPSILON || rel <= RELATIVE_TOLERANCE
  };
  assert!(
    ok,
    "{} {}: actual {} not within tolerance of expected {} (rel={}, abs_eps={})",
    kind, id, actual, expected, RELATIVE_TOLERANCE, ABSOLUTE_EPSILON
  );
}

fn measurement_from_entry(entry: &BaselineEntry) -> Measurement {
  let points: Vec<Point> = entry.points.iter().map(|p| Point::new(p.x, p.y)).collect();
  let id = entry.id.clone();
  let page_id = "1".to_string();
  let group_id = "1".to_string();

  match entry.kind.as_str() {
    "Polygon" => Measurement::Polygon {
      id,
      page_id,
      group_id,
      points,
    },
    "Polyline" => Measurement::Polyline {
      id,
      page_id,
      group_id,
      points,
    },
    "Rectangle" => {
      assert_eq!(points.len(), 2, "Rectangle must have exactly 2 points");
      Measurement::Rectangle {
        id,
        page_id,
        group_id,
        points: (points[0], points[1]),
      }
    }
    "Count" => {
      assert_eq!(points.len(), 1, "Count must have exactly 1 point");
      Measurement::Count {
        id,
        page_id,
        group_id,
        points: (points[0],),
      }
    }
    _ => panic!("unknown kind: {}", entry.kind),
  }
}

fn scale_from_entry(entry: &BaselineEntry) -> Scale {
  let unit = parse_unit(&entry.scale.unit);
  Scale::Default {
    id: "scale-1".to_string(),
    page_id: "1".to_string(),
    scale: ScaleDefinition {
      pixel_distance: entry.scale.pixel_distance,
      real_distance: entry.scale.real_distance,
      unit,
    },
  }
}

#[test]
fn golden_baseline_accuracy() {
  let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test_data/baseline.json");
  let contents = std::fs::read_to_string(&path).expect("read baseline.json");
  let entries: Vec<BaselineEntry> = serde_json::from_str(&contents).expect("parse baseline.json");

  for entry in entries {
    let measurement = measurement_from_entry(&entry);
    let scale = scale_from_entry(&entry);
    let output_unit = parse_unit(&entry.output_unit);

    let measurement_wrapper = MeasurementWrapper::default(measurement);
    measurement_wrapper.set_scale(scale);
    if let Some(expected_count) = entry.expected.count {
      let count = measurement_wrapper.get_count();
      assert_eq!(count, expected_count);
      // assert_within_tolerance(count, expected_count, "count", &entry.id);
      continue;
    }

    if let Some(expected_area) = entry.expected.area {
      let actual_area = measurement_wrapper.get_area().expect("area");
      assert_within_tolerance(
        actual_area.get_converted_value(output_unit),
        expected_area,
        "area",
        &entry.id,
      );
    }

    if let Some(expected_length) = entry.expected.length {
      let actual_length = measurement_wrapper.get_length().expect("length").unwrap();
      assert_within_tolerance(
        actual_length.get_converted_value(output_unit),
        expected_length,
        "length",
        &entry.id,
      );
    }
  }
}
