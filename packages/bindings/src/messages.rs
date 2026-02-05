use serde::{Deserialize, Serialize};
use takeoff_core::measurement::Measurement;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Message {
  MeasurementAdded(Measurement),
  UpdateMeasurement(Measurement),
  Test(String),
}
