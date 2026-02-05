use crate::measurement::MeasurementWrapper;
use dashmap::DashMap;

use napi_derive::napi;
use std::collections::HashMap;
use std::sync::Arc;
use takeoff_core::group::Group;
use takeoff_core::measurement::Measurement;
use takeoff_core::page::Page;
use takeoff_core::scale::Scale;
use takeoff_core::state::StateOptions;
#[napi]
#[derive(Debug, Clone)]
pub struct TakeoffStateHandler {
  pages: HashMap<String, Page>,
  groups: HashMap<String, Group>,
  measurements: Arc<DashMap<String, MeasurementWrapper>>,
  scales: HashMap<String, Scale>,
}

#[napi]
impl TakeoffStateHandler {
  #[napi(constructor)]
  /// Creates a new state.
  ///
  /// # Arguments
  ///
  /// * `options` - The options for the state.
  ///
  /// # Returns
  ///
  /// * `State` - The new state.
  pub fn new(options: StateOptions) -> Self {
    let pages = options
      .pages
      .into_iter()
      .map(|page| (page.id.clone(), page))
      .collect();
    let groups = options
      .groups
      .into_iter()
      .map(|group| (group.id.clone(), group))
      .collect();
    let scales = options
      .scales
      .into_iter()
      .map(|scale| (scale.id(), scale))
      .collect();
    let measurements = Arc::new(DashMap::from_iter(options.measurements.into_iter().map(
      |measurement| {
        (
          measurement.id().to_string(),
          MeasurementWrapper::new(measurement),
        )
      },
    )));

    let state = Self {
      pages,
      groups,
      measurements,
      scales,
    };

    state.compute_measurements();
    state
  }

  fn compute_measurements(&self) {
    for mut measurement in self.measurements.iter_mut() {
      let page_id = measurement.page_id();
      let scales = self.get_page_scales(page_id);
      measurement.calculate_scale(scales.into_iter().cloned().collect());
    }
  }

  fn compute_measurement(&self, measurement_id: &str) {
    let measurement = self.measurements.get_mut(measurement_id);
    if let Some(mut measurement) = measurement {
      println!("computing measurement: {:?}", measurement.id());
      let page_id = measurement.page_id();
      let scales = self.get_page_scales(page_id);
      measurement.calculate_scale(scales.into_iter().cloned().collect());
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

  fn get_page_scales(&self, page_id: &str) -> Vec<&Scale> {
    let scales = self
      .scales
      .values()
      .filter(|scale| scale.page_id() == page_id)
      .collect::<Vec<&Scale>>();
    scales
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
  pub fn upsert_page(&mut self, page: Page) -> Option<Page> {
    self.pages.insert(page.id.clone(), page)
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
  pub fn upsert_group(&mut self, group: Group) -> Option<Group> {
    self.groups.insert(group.id.clone(), group)
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
  pub fn upsert_measurement(&mut self, measurement: Measurement) -> Option<Measurement> {
    let id = measurement.id().to_string();
    let res = self.measurements.insert(
      measurement.id().to_string(),
      MeasurementWrapper::new(measurement),
    );
    self.compute_measurement(&id);

    if let Some(measurement) = res {
      return Some(measurement.get_measurement());
    }
    None
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
  pub fn upsert_scale(&mut self, scale: Scale) -> Option<Scale> {
    self.scales.insert(scale.id(), scale)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use takeoff_core::coords::Point;
  use takeoff_core::measurement::Measurement::*;
  use takeoff_core::scale::Scale::*;
  use takeoff_core::scale::ScaleDefinition;
  use takeoff_core::unit::Unit;

  #[test]
  fn test_find_measurement_scale() {
    let mut state = TakeoffStateHandler::new(StateOptions {
      pages: vec![],
      groups: vec![],
      measurements: vec![],
      scales: vec![],
    });
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
        Point::new(0.1, 0.1),
        Point::new(1.0, 0.1),
        Point::new(1.0, 1.0),
        Point::new(0.1, 1.0),
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
    let measurement = measurement;
    assert_eq!(
      measurement.get_scale(),
      Some(&Area {
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
  }
}
