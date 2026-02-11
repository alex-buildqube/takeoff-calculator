use crate::group::GroupWrapper;
use crate::measurement::MeasurementWrapper;
use anyhow::Result;
use dashmap::DashMap;
use napi_derive::napi;
use std::sync::Arc;
use takeoff_core::group::Group;
use takeoff_core::measurement::Measurement;
use takeoff_core::page::Page;
use takeoff_core::scale::Scale;
use takeoff_core::state::StateOptions;
#[napi]
#[derive(Debug, Clone, Default)]
pub struct TakeoffStateHandler {
  pages: Arc<DashMap<String, Page>>,
  groups: Arc<DashMap<String, GroupWrapper>>,
  measurements: Arc<DashMap<String, MeasurementWrapper>>,
  scales: Arc<DashMap<String, Scale>>,
}

#[napi]
impl TakeoffStateHandler {
  /// Creates a new state.
  ///
  /// # Arguments
  ///
  /// * `options` - The options for the state.
  ///
  /// # Returns
  ///
  /// * `State` - The new state.
  #[napi(constructor)]
  pub fn new(options: Option<StateOptions>) -> Self {
    let state = Self {
      pages: Arc::new(DashMap::new()),
      groups: Arc::new(DashMap::new()),
      measurements: Arc::new(DashMap::new()),
      scales: Arc::new(DashMap::new()),
    };

    if let Some(options) = options {
      state.add_initial_options(options);
    }
    state.compute_measurements();
    state
  }

  #[napi]
  pub fn get_measurements_by_group_id(&self, group_id: String) -> Vec<MeasurementWrapper> {
    self
      .measurements
      .iter()
      .filter(|entry| entry.value().get_group_id() == group_id)
      .map(|entry| entry.value().clone())
      .collect()
  }

  /// Get the measurements by page id.
  ///
  /// # Arguments
  ///
  /// * `page_id` - The id of the page.
  ///
  /// # Returns
  ///
  /// * `Vec<MeasurementWrapper>` - The measurements that are on the page.
  #[napi]
  pub fn get_measurements_by_page_id(&self, page_id: String) -> Vec<MeasurementWrapper> {
    self
      .measurements
      .iter()
      .filter(|entry| entry.value().page_id() == page_id)
      .map(|entry| entry.value().clone())
      .collect()
  }

  fn add_initial_options(&self, options: StateOptions) {
    for page in options.pages {
      self.pages.insert(page.id.clone(), page);
    }
    for scale in options.scales {
      self.scales.insert(scale.id(), scale);
    }
    for group in options.groups {
      self.groups.insert(
        group.id.clone(),
        GroupWrapper::new(group, Arc::new(self.clone())),
      );
    }
    for measurement in options.measurements {
      self.measurements.insert(
        measurement.id().to_string(),
        MeasurementWrapper::new(measurement, Arc::new(self.clone())),
      );
    }
  }

  fn compute_measurements(&self) {
    for measurement in self.measurements.iter() {
      measurement.calculate_scale();
    }
  }

  fn compute_page(&self, page_id: &str) {
    let measurements = self
      .measurements
      .iter()
      .filter(|entry| entry.value().page_id() == page_id)
      .map(|entry| entry.value().clone())
      .collect::<Vec<MeasurementWrapper>>();
    for measurement in measurements {
      measurement.calculate_scale();
    }
  }

  #[napi]
  /// Get the scale for a measurement.
  ///
  /// # Arguments
  ///
  /// * `measurement_id` - The id of the measurement.
  ///
  /// # Returns
  ///
  /// * `None` - If the measurement was not found.
  /// * `Some(scale)` - If the scale was found.
  pub fn get_measurement_scale(&self, measurement_id: String) -> Option<Scale> {
    let measurement = self.measurements.get_mut(&measurement_id);
    if let Some(measurement) = measurement {
      if let Some(scale) = measurement.get_scale() {
        return Some(scale.clone());
      }
      // return Some(measurement.get_scale().clone());
    }
    None
    // self.find_measurement_scale(measurement)
  }

  pub fn get_page_scales(&self, page_id: &str) -> Vec<Scale> {
    self
      .scales
      .iter()
      .filter(|entry| entry.value().page_id() == page_id)
      .map(|entry| entry.value().clone())
      .collect::<Vec<Scale>>()
  }

  #[napi]
  /// Inserts or updates a page in the state.
  ///
  /// # Arguments
  ///
  /// * `page` - The page to insert or update.
  ///
  /// # Returns
  ///
  /// * `None` - If the page was not found.
  /// * `Some(page)` - If the page was found and updated.
  pub fn upsert_page(&self, page: Page) -> Option<Page> {
    self.pages.insert(page.id.clone(), page)
  }

  #[napi]
  pub fn remove_page(&self, page_id: String) -> Option<Page> {
    self.pages.remove(&page_id).map(|(_, page)| page)
  }

  #[napi]
  pub fn get_group(&self, group_id: String) -> Option<GroupWrapper> {
    self
      .groups
      .get(&group_id)
      .map(|entry| entry.value().clone())
  }

  #[napi]
  /// Inserts or updates a group in the state.
  ///
  /// # Arguments
  ///
  /// * `group` - The group to insert or update.
  ///
  /// # Returns
  ///
  /// * `None` - If the group was not found.
  /// * `Some(group)` - If the group was found and updated.
  pub fn upsert_group(&self, group: Group) -> Option<Group> {
    let group_clone = group.clone();
    self.groups.insert(
      group.id.clone(),
      GroupWrapper::new(group, Arc::new(self.clone())),
    );
    Some(group_clone)
  }

  #[napi]
  /// Removes a group from the state.
  ///
  /// # Arguments
  ///
  /// * `group_id` - The id of the group to remove.
  ///
  /// # Returns
  /// * `None` - If the group was not found.
  /// * `Some(group)` - If the group was found and removed.
  pub fn remove_group(&self, group_id: String) -> Option<Group> {
    let res = self.groups.remove(&group_id);
    if let Some((_, group)) = res {
      // also remove any measurements tied to this group
      let to_remove: Vec<String> = self
        .measurements
        .iter()
        .filter(|entry| entry.value().get_group_id() == group_id)
        .map(|entry| entry.key().clone())
        .collect();
      for mid in to_remove {
        self.remove_measurement(mid);
      }
      return Some(group.get_group());
    }

    None
  }

  #[napi]
  /// Inserts or updates a measurement in the state.
  ///
  /// # Arguments
  ///
  /// * `measurement` - The measurement to insert or update.
  ///
  /// # Returns
  ///
  /// * `None` - If the measurement was not found.
  /// * `Some(measurement)` - If the measurement was found and updated.
  pub fn upsert_measurement(&self, measurement: Measurement) -> Option<Measurement> {
    let id = measurement.id().to_string();

    if let Some(prev_measurement) = self.measurements.get(&id) {
      prev_measurement.set_measurement(measurement.clone());
      return Some(prev_measurement.get_measurement());
    }

    let res = self.measurements.insert(
      measurement.id().to_string(),
      MeasurementWrapper::new(measurement.clone(), Arc::new(self.clone())),
    );
    self.compute_measurement(&id);
    let _ = self.compute_group(measurement.group_id());

    if let Some(measurement) = res {
      return Some(measurement.get_measurement());
    }
    None
  }

  #[napi]
  /// Removes a measurement from the state.
  ///
  /// # Arguments
  ///
  /// * `measurement_id` - The id of the measurement to remove.
  ///
  /// # Returns
  ///
  /// * `None` - If the measurement was not found.
  /// * `Some(measurement)` - If the measurement was found and removed.
  pub fn remove_measurement(&self, measurement_id: String) -> Option<Measurement> {
    let res = self.measurements.remove(&measurement_id);
    if let Some((_, measurement)) = res {
      // Ignore recomputation errors - they will be handled when group values are accessed
      let _ = self.compute_group(&measurement.get_group_id());
      return Some(measurement.get_measurement());
    }
    None
  }

  #[napi]
  pub fn get_measurement(&self, measurement_id: String) -> Option<MeasurementWrapper> {
    self
      .measurements
      .get(&measurement_id)
      .map(|entry| entry.value().clone())
  }

  #[napi]
  /// Inserts or updates a scale in the state.
  ///
  /// # Arguments
  ///
  /// * `scale` - The scale to insert or update.
  ///
  /// # Returns
  ///
  /// * `None` - If the scale was not found.
  /// * `Some(scale)` - If the scale was found and updated.
  pub fn upsert_scale(&self, scale: Scale) -> Option<Scale> {
    let page_id = scale.page_id();
    let res = self.scales.insert(scale.id(), scale);
    self.compute_page(&page_id);
    res
  }

  #[napi]
  /// Removes a scale from the state.
  ///
  /// # Arguments
  ///
  /// * `scale_id` - The id of the scale to remove.
  ///
  /// # Returns
  /// * `None` - If the scale was not found.
  /// * `Some(scale)` - If the scale was found and removed.
  pub fn remove_scale(&self, scale_id: String) -> Option<Scale> {
    let scale = self.scales.remove(&scale_id);
    if let Some((_, scale)) = scale {
      self.compute_page(&scale.page_id());
      return Some(scale);
    }
    None
  }

  #[napi]
  /// Get the measurements that are missing a scale.
  ///
  /// # Returns
  ///
  /// * `Vec<MeasurementWrapper>` - The measurements that are missing a scale.
  pub fn get_measurements_missing_scale(&self) -> Vec<MeasurementWrapper> {
    self
      .measurements
      .iter()
      .filter(|entry| entry.value().get_scale().is_none())
      .map(|entry| entry.value().clone())
      .collect()
  }
}

#[napi]
#[cfg(not(target_family = "wasm"))]
impl TakeoffStateHandler {
  fn compute_measurement(&self, measurement_id: &str) {
    let measurement = self.measurements.get(measurement_id);
    if let Some(measurement) = measurement {
      std::thread::scope(|s| {
        s.spawn(|| {
          measurement.calculate_scale();
        });
      });
    }
  }

  pub fn compute_group(&self, group_id: &str) -> Result<()> {
    let group = self.groups.get(group_id);
    if let Some(group) = group {
      std::thread::scope(|s| {
        s.spawn(|| {
          // Ignore recomputation errors - they will be handled when group values are accessed
          let _ = group.recompute_measurements();
        });
      });
    }
    Ok(())
  }
}

#[napi]
#[cfg(target_family = "wasm")]
impl TakeoffStateHandler {
  fn compute_measurement(&self, measurement_id: &str) {
    let measurement = self.measurements.get(measurement_id);
    if let Some(measurement) = measurement {
      measurement.calculate_scale();
    }
  }

  pub fn compute_group(&self, group_id: &str) -> Result<()> {
    let group = self.groups.get(group_id);
    if let Some(group) = group {
      group.recompute_measurements().unwrap();
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use takeoff_core::coords::Point;
  use takeoff_core::group::MeasurementType;
  use takeoff_core::measurement::Measurement::*;
  use takeoff_core::scale::Scale::*;
  use takeoff_core::scale::ScaleDefinition;
  use takeoff_core::unit::Unit;

  #[test]
  fn test_find_measurement_scale() {
    let state = TakeoffStateHandler::new(Some(StateOptions {
      pages: vec![],
      groups: vec![],
      measurements: vec![],
      scales: vec![],
    }));
    state.upsert_scale(Area {
      id: "1".to_string(),
      page_id: "1".to_string(),
      bounding_box: (Point::new(0.0, 0.0), Point::new(1.0, 1.0)),
      scale: ScaleDefinition {
        pixel_distance: 1.0,
        real_distance: 1.0,
        unit: Unit::Meters,
      },
    });
    state.upsert_scale(Default {
      id: "2".to_string(),
      page_id: "1".to_string(),
      scale: ScaleDefinition {
        pixel_distance: 1.0,
        real_distance: 1.0,
        unit: Unit::Meters,
      },
    });
    let measurement = Polygon {
      id: "1".to_string(),
      page_id: "1".to_string(),
      group_id: "1".to_string(),
      points: vec![
        Point::new(0.5, 0.5),
        Point::new(1.0, 0.5),
        Point::new(1.0, 1.0),
        Point::new(0.5, 1.0),
      ],
    };
    state.upsert_measurement(measurement.clone());
    let scale = state.get_measurement_scale(measurement.id().to_string());
    assert_eq!(
      scale,
      Some(Area {
        id: "1".to_string(),
        page_id: "1".to_string(),
        bounding_box: (Point::new(0.0, 0.0), Point::new(1.0, 1.0)),
        scale: ScaleDefinition {
          pixel_distance: 1.0,
          real_distance: 1.0,
          unit: Unit::Meters,
        },
      })
    );

    let measurement = state
      .measurements
      .get(&measurement.id().to_string())
      .unwrap();
    let measurement_clone = measurement.clone();
    assert_eq!(
      measurement.get_scale(),
      Some(Area {
        id: "1".to_string(),
        page_id: "1".to_string(),
        bounding_box: (Point::new(0.0, 0.0), Point::new(1.0, 1.0)),
        scale: ScaleDefinition {
          pixel_distance: 1.0,
          real_distance: 1.0,
          unit: Unit::Meters,
        },
      })
    );
    assert_eq!(
      measurement_clone
        .get_area()
        .unwrap()
        .get_converted_value(Unit::Meters),
      0.25
    );

    let group = Group {
      id: "1".to_string(),
      name: None,
      measurement_type: MeasurementType::Area,
    };
    state.upsert_group(group);
    let group = state.groups.get("1").unwrap();
    let group_clone = group.clone();
    assert_eq!(
      group.get_area().unwrap().get_converted_value(Unit::Meters),
      0.25
    );

    state.upsert_measurement(Measurement::Rectangle {
      id: "12".to_string(),
      page_id: "1".to_string(),
      group_id: "1".to_string(),
      points: (Point::new(0.0, 0.0), Point::new(1.0, 1.0)),
    });

    let initial_group_area = {
      group_clone
        .get_area()
        .unwrap()
        .get_converted_value(Unit::Meters)
    };
    println!("initial_group_area: {}", initial_group_area);

    let measurement = state.remove_measurement("12".to_string());
    assert!(measurement.is_some());
    drop(measurement);

    let group = state.get_group("1".to_string());
    assert!(group.is_some());
    let group_area = group
      .unwrap()
      .get_area()
      .unwrap()
      .get_converted_value(Unit::Meters);
    assert_eq!(group_area, 0.25);

    // let group_removed = state.remove_group("1".to_string());
  }
  #[test]
  fn test_remove_group() {
    let state = TakeoffStateHandler::new(Some(StateOptions {
      pages: vec![],
      groups: vec![],
      measurements: vec![],
      scales: vec![],
    }));
    let group = Group {
      id: "1".to_string(),
      name: None,
      measurement_type: MeasurementType::Area,
    };
    state.upsert_group(group);
    // let group = state.groups.get("1").unwrap();

    let group_removed = state.remove_group("1".to_string());
    assert!(group_removed.is_some());
    let group = state.get_group("1".to_string());
    assert!(group.is_none());
  }
}
