use crate::coords::Point;
use crate::unit::Unit;
use geo::Contains;
use geo::{Coord, Geometry, Polygon as GeoPolygon, Rect};
use napi_derive::napi;
use serde::{Deserialize, Serialize};

// #[napi(string_enum)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
// pub enum ScaleType {
//   Area,
//   Default,
// }

#[napi(object)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScaleDefinition {
  pub pixel_distance: f64,
  pub real_distance: f64,
  pub unit: Unit,
}

#[napi(discriminant = "type")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Scale {
  Area {
    id: String,
    page_id: String,
    scale: ScaleDefinition,
    bounding_box: (Point, Point),
  },
  Default {
    id: String,
    page_id: String,
    scale: ScaleDefinition,
  },
}

impl Scale {
  pub fn id(&self) -> String {
    match self {
      Scale::Area { id, .. } => id.clone(),
      Scale::Default { id, .. } => id.clone(),
    }
  }

  pub fn page_id(&self) -> String {
    match self {
      Scale::Area { page_id, .. } => page_id.clone(),
      Scale::Default { page_id, .. } => page_id.clone(),
    }
  }

  pub fn bounding_box_to_polygon(&self) -> Option<GeoPolygon<f64>> {
    match self {
      Scale::Area { bounding_box, .. } => {
        let start: Coord<f64> = bounding_box.0.into();
        let end: Coord<f64> = bounding_box.1.into();
        let rect = Rect::new(start, end);
        Some(rect.to_polygon())
      }
      _ => None,
    }
  }

  pub fn is_in_bounding_box(&self, geometry: &Geometry<f64>) -> bool {
    match self {
      Scale::Area { .. } => {
        let polygon = self.bounding_box_to_polygon().unwrap();
        polygon.contains(geometry)
      }
      _ => false,
    }
  }
}
