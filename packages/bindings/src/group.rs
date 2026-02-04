use napi_derive::napi;
use serde::{Deserialize, Serialize};

#[napi(string_enum)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeasurementType {
  Area,
  Linear,
  Count,
}

#[napi(object)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group {
  pub id: String,
  pub name: Option<String>,
  //   pub measurements: Vec<Measurement>,
  pub measurement_type: MeasurementType,
}
