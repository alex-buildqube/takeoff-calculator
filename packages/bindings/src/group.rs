use crate::measurement::MeasurementWrapper;
use crate::state::TakeoffStateHandler;
use crate::utils::lock_mutex;
use anyhow::Result;
use napi_derive::napi;
use std::sync::{Arc, Mutex};
use takeoff_core::error::TakeoffResult;
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
    let area = measurements
      .iter()
      .filter_map(|measurement| measurement.get_area_value().unwrap_or(None))
      .reduce(|a, b| a + b);
    Ok(area)
  }

  fn calculate_length(&self, measurements: &[MeasurementWrapper]) -> TakeoffResult<Option<Length>> {
    let mut length_opt = None;
    for measurement in measurements {
      if let Ok(Some(length)) = measurement.get_length_value() {
        length_opt = Some(match length_opt {
          Some(acc) => acc + length,
          None => length,
        });
      }
    }
    Ok(length_opt)
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

  /// Recompute all measurements for this group.
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// - Mutex lock fails (poisoned mutex)
  /// - Area calculation fails
  /// - Length calculation fails
  pub fn recompute_measurements(&self) -> Result<()> {
    let measurements = self
      .state
      .get_measurements_by_group_id(self.id().to_string());

    {
      *lock_mutex(self.area.lock(), "area")? = self.calculate_area(&measurements)?;
    }

    {
      *lock_mutex(self.length.lock(), "length")? = self.calculate_length(&measurements)?;
    }

    {
      *lock_mutex(self.points.lock(), "points")? = self.calculate_points(&measurements);
    }

    {
      *lock_mutex(self.count.lock(), "count")? = self.calculate_count(&measurements);
    }

    Ok(())
  }

  #[napi(getter)]
  /// Get the id of the group.
  pub fn id(&self) -> &str {
    &self.group.id
  }

  #[napi(getter)]
  /// Get the area for this group.
  ///
  /// Returns `None` if the area has not been computed or if the mutex is poisoned.
  pub fn get_area(&self) -> Option<UnitValue> {
    if let Ok(area) = self.area.lock() {
      if let Some(area) = area.as_ref() {
        return Some(UnitValue::from_area(*area));
      }
    }
    None
  }

  #[napi(getter)]
  /// Get the length for this group.
  ///
  /// Returns `None` if the length has not been computed or if the mutex is poisoned.
  pub fn get_length(&self) -> Option<UnitValue> {
    if let Ok(length) = lock_mutex(self.length.lock(), "length") {
      if let Some(length) = length.as_ref() {
        return Some(UnitValue::from_length(*length));
      }
    }
    None
  }

  #[napi(getter)]
  /// Get the points count for this group.
  ///
  /// Returns `None` if the points count has not been computed or if the mutex is poisoned.
  pub fn get_points(&self) -> Option<f64> {
    lock_mutex(self.points.lock(), "points")
      .ok()
      .and_then(|p| *p)
  }

  #[napi(getter)]
  /// Get the count for this group.
  ///
  /// Returns `None` if the count has not been computed or if the mutex is poisoned.
  pub fn get_count(&self) -> Option<f64> {
    lock_mutex(self.count.lock(), "count").ok().and_then(|c| *c)
  }

  #[napi(getter)]
  pub fn get_group(&self) -> Group {
    self.group.clone()
  }
}
