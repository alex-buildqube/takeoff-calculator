use std::collections::HashMap;

use napi_derive::napi;
use serde::{Deserialize, Serialize};

use crate::group::Group;
use crate::measurement::Measurement;
use crate::measurement::MeasurementWrapper;
use crate::page::Page;
use crate::scale::Scale;

#[napi(object)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateOptions {
  pub pages: Vec<Page>,
  pub groups: Vec<Group>,
  pub measurements: Vec<Measurement>,
  pub scales: Vec<Scale>,
}

#[napi]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct State {
  pages: HashMap<String, Page>,
  groups: HashMap<String, Group>,
  measurements: HashMap<String, MeasurementWrapper>,
  scales: HashMap<String, Scale>,
}

#[napi]
impl State {
  #[napi(constructor)]
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
    let measurements = options
      .measurements
      .into_iter()
      .map(|measurement| {
        (
          measurement.id().to_string(),
          MeasurementWrapper::new(measurement),
        )
      })
      .collect();
    let scales = options
      .scales
      .into_iter()
      .map(|scale| (scale.id(), scale))
      .collect();

    Self {
      pages,
      groups,
      measurements,
      scales,
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
    let measurement = self.measurements.get(&measurement_id)?;
    self
      .find_measurement_scale(&measurement.measurement)
      .cloned()
  }

  fn find_measurement_scale(&self, measurement: &Measurement) -> Option<&Scale> {
    let page_id = measurement.page_id();
    let scales = self.get_page_scales(page_id);

    if scales.is_empty() {
      return None;
    }

    let mut current_scale: Option<&Scale> = None;
    for scale in scales {
      if matches!(scale, Scale::Area { .. }) {
        if scale.is_in_bounding_box(&measurement.to_geometry()) {
          return Some(scale);
        } else {
          continue;
        }
      } else {
        current_scale = Some(scale);
      }
    }
    current_scale
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
    let measurement = self.measurements.insert(
      measurement.id().to_string(),
      MeasurementWrapper::new(measurement),
    );
    measurement.map(|m| m.measurement)
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
  use crate::coords::Point;
  use crate::measurement::Measurement::*;
  use crate::scale::Scale::*;
  use crate::scale::ScaleDefinition;
  use crate::unit::Unit;

  #[test]
  fn test_find_measurement_scale() {
    let mut state = State::new(StateOptions {
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
    let scale = state.find_measurement_scale(&measurement);
    assert_eq!(
      scale,
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
