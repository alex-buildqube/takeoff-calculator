use serde::{Deserialize, Serialize};
use takeoff_core::measurement::Measurement;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementWrapper {
  pub measurement: Measurement,
}

impl MeasurementWrapper {
  pub fn new(measurement: Measurement) -> Self {
    Self { measurement }
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
    self.measurement.area()
  }

  pub fn raw_perimeter(&self) -> f64 {
    self.measurement.perimeter()
  }
}
