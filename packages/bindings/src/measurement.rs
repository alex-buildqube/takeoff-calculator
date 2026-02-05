use napi_derive::napi;
use serde::{Deserialize, Serialize};
use takeoff_core::scale::Scale;
use takeoff_core::{measurement::Measurement, unit::Unit};
use uom::si::f32::{Area, Length};

#[napi(js_name = "MeasurementCalculator")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementWrapper {
  measurement: Measurement,

  scale: Option<Scale>,
  area: Option<Area>,
  length: Option<Length>,
  points: f64,
}

#[napi]
impl MeasurementWrapper {
  #[napi(constructor)]
  pub fn new(measurement: Measurement) -> Self {
    let points = match measurement.clone() {
      Measurement::Count { .. } => 1,
      Measurement::Polygon { points, .. } => points.len(),
      Measurement::Polyline { points, .. } => points.len(),
      Measurement::Rectangle { .. } => 2,
    };
    Self {
      measurement,
      scale: None,
      area: None,
      length: None,
      points: points as f64,
    }
  }

  fn calculate_area(&self) -> Option<Area> {
    if let Some(scale) = self.get_scale() {
      let scale_ratio = scale.ratio();
      println!("scale_ratio: {:?}", scale_ratio);
      let area = self.raw_area() / (scale_ratio * scale_ratio);
      let res = scale.get_unit().get_area_unit(area as f32);
      return Some(res);
    }
    None
  }

  #[napi]
  pub fn get_measurement(&self) -> Measurement {
    self.measurement.clone()
  }

  pub fn get_area(&mut self) -> Option<Area> {
    if self.area.is_none() {
      self.area = self.calculate_area();
    }
    self.area
  }

  pub fn calculate_scale(&mut self, scales: Vec<Scale>) -> Option<Scale> {
    let mut current_scale: Option<Scale> = None;
    for scale in scales {
      if matches!(scale, Scale::Area { .. }) {
        if scale.is_in_bounding_box(&self.measurement.to_geometry()) {
          println!("setting scale: {:?}", scale);
          self.set_scale(scale.clone());
          return Some(scale);
        }
      } else {
        // println!("setting scale: {:?}", scale);
        current_scale = Some(scale.clone());
      }
    }
    if let Some(scale) = current_scale {
      self.set_scale(scale.clone());
      return Some(scale);
    }
    None
  }

  #[napi]
  pub fn convert_area(&self, unit: Unit) -> Option<f32> {
    if let Some(area) = self.calculate_area() {
      return Some(unit.convert_area_to_unit(area));
    }
    None
  }

  pub fn get_length(&mut self) -> Option<Length> {
    if self.length.is_none() {
      self.length = self.calculate_length();
    }
    self.length
  }

  fn calculate_length(&self) -> Option<Length> {
    if let Some(scale) = self.get_scale() {
      let scale_ratio = scale.ratio();
      let length = self.raw_perimeter() / scale_ratio;
      let res = scale.get_unit().get_unit(length as f32);
      return Some(res);
    }
    None
  }

  #[napi]
  pub fn convert_length(&self, unit: Unit) -> Option<f32> {
    if let Some(length) = self.calculate_length() {
      return Some(unit.convert_length_to_unit(length));
    }
    None
  }

  pub fn recompute_measurements(&mut self) {
    self.area = self.calculate_area();
    self.length = self.calculate_length();
  }

  pub fn set_scale(&mut self, scale: Scale) {
    self.scale = Some(scale);
    self.recompute_measurements();
  }

  pub fn get_scale(&self) -> Option<&Scale> {
    self.scale.as_ref()
  }

  pub fn id(&self) -> &str {
    self.measurement.id()
  }

  pub fn page_id(&self) -> &str {
    self.measurement.page_id()
  }

  pub fn group_id(&self) -> &str {
    self.measurement.group_id()
  }

  pub fn raw_area(&self) -> f64 {
    self.measurement.pixel_area()
  }

  pub fn raw_perimeter(&self) -> f64 {
    self.measurement.pixel_perimeter()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use takeoff_core::{coords::Point, scale::ScaleDefinition, unit::Unit};
  use uom::si::area::square_meter;

  #[test]
  fn test_calculate_area() {
    let measurement = Measurement::Rectangle {
      id: "1".to_string(),
      page_id: "1".to_string(),
      group_id: "1".to_string(),
      points: (Point::new(0.0, 0.0), Point::new(100.0, 50.0)),
    };

    assert!(measurement.pixel_area() == 5000.0);
    let mut measurement_wrapper = MeasurementWrapper::new(measurement);
    measurement_wrapper.set_scale(Scale::Default {
      id: "1".to_string(),
      page_id: "1".to_string(),
      scale: ScaleDefinition {
        pixel_distance: 100.0,
        real_distance: 2.0,
        unit: Unit::Meters,
      },
    });
    let area = measurement_wrapper.calculate_area().unwrap();
    println!("area: {:?}", area);
    assert_eq!(area.get::<square_meter>(), 2.0);
    assert_eq!(measurement_wrapper.convert_area(Unit::Meters).unwrap(), 2.0);
    assert_eq!(
      measurement_wrapper.convert_length(Unit::Meters).unwrap(),
      6.0
    );
  }

  #[test]
  fn test_calculate_without_scale() {
    let measurement = Measurement::Rectangle {
      id: "1".to_string(),
      page_id: "1".to_string(),
      group_id: "1".to_string(),
      points: (Point::new(0.0, 0.0), Point::new(100.0, 50.0)),
    };
    let measurement_wrapper = MeasurementWrapper::new(measurement);
    assert_eq!(measurement_wrapper.raw_area(), 5000.0);
    assert_eq!(measurement_wrapper.raw_perimeter(), 300.0);
    assert_eq!(measurement_wrapper.convert_area(Unit::Meters), None);
    assert_eq!(measurement_wrapper.convert_length(Unit::Meters), None);
  }
}
