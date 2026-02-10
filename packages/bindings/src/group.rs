use crate::measurement::MeasurementWrapper;
use crate::state::TakeoffStateHandler;
use anyhow::Result;
use napi_derive::napi;
use std::sync::{Arc, Mutex};
use takeoff_core::group::Group;
use takeoff_core::unit::UnitValue;

use uom::si::f32::{Area, Length};

#[napi]
#[derive(Debug, Clone)]
pub struct GroupWrapper {
  group: Group,
  area: Arc<Mutex<Option<Area>>>,
  length: Arc<Mutex<Option<Length>>>,
  points: Arc<Mutex<Option<f64>>>,
  count: Arc<Mutex<Option<f64>>>,

  // #[serde(skip)]
  state: Arc<TakeoffStateHandler>,
}

#[napi]
impl GroupWrapper {
  pub fn new(group: Group, state: Arc<TakeoffStateHandler>) -> Self {
    let res = Self {
      group,
      state,
      area: Arc::new(Mutex::new(None)),
      length: Arc::new(Mutex::new(None)),
      points: Arc::new(Mutex::new(None)),
      count: Arc::new(Mutex::new(None)),
    };
    let _ = res.recompute_measurements();
    res
  }

  fn calculate_area(&self, measurements: &[MeasurementWrapper]) -> Result<Option<Area>> {
    // let measurements = self.get_measurements();
    // println!("measurements: {:?}", measurements);
    let area = measurements
      .iter()
      .filter_map(|measurement| measurement.get_area_value().unwrap_or(None))
      // .filter(|area| area.is_some())
      // .map(|area| area.unwrap())
      .reduce(|a, b| a + b);
    Ok(area)
  }

  fn calculate_length(&self, measurements: &[MeasurementWrapper]) -> Result<Option<Length>> {
    let length = measurements
      .iter()
      .filter_map(|measurement| measurement.get_length_value())
      .reduce(|a, b| a + b);
    Ok(length)
  }

  fn calculate_points(&self, measurements: &[MeasurementWrapper]) -> Option<f64> {
    let points = measurements
      .iter()
      .map(|measurement| measurement.get_points())
      .reduce(|a, b| a + b);

    points
  }
  fn calculate_count(&self, measurements: &[MeasurementWrapper]) -> Option<f64> {
    Some(measurements.len() as f64)
  }

  pub fn recompute_measurements(&self) -> Result<()> {
    let measurements = self
      .state
      .get_measurements_by_group_id(self.id().to_string());

    {
      *self.area.lock().unwrap() = self.calculate_area(&measurements)?;
    }

    {
      *self.length.lock().unwrap() = self.calculate_length(&measurements)?;
    }

    {
      *self.points.lock().unwrap() = self.calculate_points(&measurements);
    }

    {
      *self.count.lock().unwrap() = self.calculate_count(&measurements);
    }

    Ok(())
  }

  //   #[napi]
  //   pub fn get_measurements(&self) -> Vec<MeasurementWrapper> {
  //     println!("getting measurements: {:?}", self.group);
  //     let res = self.state.get_measurements_by_group_id(self.id());

  //     // println!("measurements: {:?}", res);
  //     res
  //   }

  #[napi(getter)]
  /// Get the id of the group.
  pub fn id(&self) -> &str {
    &self.group.id
  }

  #[napi(getter)]
  pub fn get_area(&self) -> Option<UnitValue> {
    if let Some(area) = self.area.lock().unwrap().as_ref() {
      return Some(UnitValue::from_area(*area));
    }
    None
  }

  #[napi(getter)]
  pub fn get_length(&self) -> Option<UnitValue> {
    if let Some(length) = self.length.lock().unwrap().as_ref() {
      return Some(UnitValue::from_length(*length));
    }
    None
  }

  #[napi(getter)]
  pub fn get_points(&self) -> Option<f64> {
    *self.points.lock().unwrap()
  }

  #[napi(getter)]
  pub fn get_count(&self) -> Option<f64> {
    *self.count.lock().unwrap()
  }

  #[napi(getter)]
  pub fn get_group(&self) -> Group {
    self.group.clone()
  }
}
