use napi_derive::napi;
use serde::{Deserialize, Serialize};
use uom::fmt::DisplayStyle::Abbreviation;
use uom::si::area::{square_centimeter, square_foot, square_inch, square_meter, square_yard};
use uom::si::f32::{Area, Length};
use uom::si::length::{centimeter, foot, inch, meter, yard};
/// Measurement units supported by the system
#[napi(string_enum)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unit {
  /// Imperial units
  Yards,
  Feet,
  Inches,
  /// Metric units
  Meters,
  Centimeters,
}

impl Unit {
  pub fn convert_length_to_unit(&self, length: Length) -> f32 {
    match self {
      Unit::Yards => length.get::<yard>(),
      Unit::Feet => length.get::<foot>(),
      Unit::Inches => length.get::<inch>(),
      Unit::Meters => length.get::<meter>(),
      Unit::Centimeters => length.get::<centimeter>(),
    }
  }
  pub fn get_unit(&self, value: f32) -> Length {
    match self {
      Unit::Yards => Length::new::<yard>(value),
      Unit::Feet => Length::new::<foot>(value),
      Unit::Inches => Length::new::<inch>(value),
      Unit::Meters => Length::new::<meter>(value),
      Unit::Centimeters => Length::new::<centimeter>(value),
    }
  }

  pub fn convert_area_to_unit(&self, area: Area) -> f32 {
    match self {
      Unit::Yards => area.get::<square_yard>(),
      Unit::Feet => area.get::<square_foot>(),
      Unit::Inches => area.get::<square_inch>(),
      Unit::Meters => area.get::<square_meter>(),
      Unit::Centimeters => area.get::<square_centimeter>(),
    }
  }

  pub fn get_area_unit(&self, value: f32) -> Area {
    match self {
      Unit::Yards => Area::new::<square_yard>(value),
      Unit::Feet => Area::new::<square_foot>(value),
      Unit::Inches => Area::new::<square_inch>(value),
      Unit::Meters => Area::new::<square_meter>(value),
      Unit::Centimeters => Area::new::<square_centimeter>(value),
    }
  }

  /// Convert a value from one unit to another
  pub fn convert(&self, value: f32, to: &Unit) -> f32 {
    let from = self.get_unit(value);

    match to {
      Unit::Yards => from.get::<yard>(),
      Unit::Feet => from.get::<foot>(),
      Unit::Inches => from.get::<inch>(),
      Unit::Meters => from.get::<meter>(),
      Unit::Centimeters => from.get::<centimeter>(),
    }
  }

  pub fn convert_area(&self, value: f32, to: &Unit) -> f32 {
    let from = self.get_area_unit(value);

    match to {
      Unit::Yards => from.get::<square_yard>(),
      Unit::Feet => from.get::<square_foot>(),
      Unit::Inches => from.get::<square_inch>(),
      Unit::Meters => from.get::<square_meter>(),
      Unit::Centimeters => from.get::<square_centimeter>(),
    }
  }

  /// Get the display string for this unit
  pub fn display(&self) -> &'static str {
    match self {
      Unit::Yards => "yd",
      Unit::Feet => "ft",
      Unit::Inches => "in",
      Unit::Meters => "m",
      Unit::Centimeters => "cm",
    }
  }

  pub fn unit_str(&self) -> &'static str {
    match self {
      Unit::Yards => "Yards",
      Unit::Feet => "Feet",
      Unit::Inches => "Inches",
      Unit::Meters => "Meters",
      Unit::Centimeters => "Centimeters",
    }
  }
}

/// Unit conversion utilities
pub struct UnitUtils;

impl UnitUtils {
  /// Convert a value from one unit to another
  pub fn convert(value: f32, from: Unit, to: Unit) -> f32 {
    from.convert(value, &to)
  }
  pub fn convert_area(value: f32, from: Unit, to: Unit) -> f32 {
    from.convert_area(value, &to)
  }

  /// Get all available units
  pub fn all_units() -> Vec<Unit> {
    vec![
      Unit::Yards,
      Unit::Feet,
      Unit::Inches,
      Unit::Meters,
      Unit::Centimeters,
    ]
  }

  /// Get imperial units
  pub fn imperial_units() -> Vec<Unit> {
    vec![Unit::Yards, Unit::Feet, Unit::Inches]
  }

  /// Get metric units
  pub fn metric_units() -> Vec<Unit> {
    vec![Unit::Meters, Unit::Centimeters]
  }
}

pub enum UnitFormatter {
  Length { unit: Unit, value: f32 },
  Area { unit: Unit, value: f32 },
}

impl UnitFormatter {
  pub fn format(&self) -> String {
    match self {
      UnitFormatter::Area {
        unit: Unit::Yards,
        value,
      } => Unit::Yards
        .get_area_unit(*value)
        .into_format_args(square_yard, Abbreviation)
        .to_string(),
      UnitFormatter::Area {
        unit: Unit::Feet,
        value,
      } => Unit::Feet
        .get_area_unit(*value)
        .into_format_args(square_foot, Abbreviation)
        .to_string(),
      UnitFormatter::Area {
        unit: Unit::Inches,
        value,
      } => Unit::Inches
        .get_area_unit(*value)
        .into_format_args(square_inch, Abbreviation)
        .to_string(),
      UnitFormatter::Area {
        unit: Unit::Meters,
        value,
      } => Unit::Meters
        .get_area_unit(*value)
        .into_format_args(square_meter, Abbreviation)
        .to_string(),
      UnitFormatter::Area {
        unit: Unit::Centimeters,
        value,
      } => Unit::Centimeters
        .get_area_unit(*value)
        .into_format_args(square_centimeter, Abbreviation)
        .to_string(),
      UnitFormatter::Length {
        unit: Unit::Yards,
        value,
      } => Unit::Yards
        .get_unit(*value)
        .into_format_args(yard, Abbreviation)
        .to_string(),
      UnitFormatter::Length {
        unit: Unit::Feet,
        value,
      } => Unit::Feet
        .get_unit(*value)
        .into_format_args(foot, Abbreviation)
        .to_string(),
      UnitFormatter::Length {
        unit: Unit::Inches,
        value,
      } => Unit::Inches
        .get_unit(*value)
        .into_format_args(inch, Abbreviation)
        .to_string(),
      UnitFormatter::Length {
        unit: Unit::Meters,
        value,
      } => Unit::Meters
        .get_unit(*value)
        .into_format_args(meter, Abbreviation)
        .to_string(),
      UnitFormatter::Length {
        unit: Unit::Centimeters,
        value,
      } => Unit::Centimeters
        .get_unit(*value)
        .into_format_args(centimeter, Abbreviation)
        .to_string(),
    }
  }
}

// #[napi(discriminant = "type")]
// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub enum UnitValue {
//   Area { value: Area },
//   Length { value: Length },
// }

#[napi(string_enum)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitMagnitude {
  Area,
  Length,
}

#[napi]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitValue {
  area: Option<Area>,
  length: Option<Length>,
  pub magnitude: UnitMagnitude,
}

#[napi]
impl UnitValue {
  pub fn new(value: f32, unit: Unit, magnitude: UnitMagnitude) -> Self {
    match magnitude {
      UnitMagnitude::Area => Self {
        area: Some(unit.get_area_unit(value)),
        length: None,
        magnitude,
      },
      UnitMagnitude::Length => Self {
        area: None,
        length: Some(unit.get_unit(value)),
        magnitude,
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_format() {
    let formatter = UnitFormatter::Length {
      unit: Unit::Meters,
      value: 1.0,
    };
    assert_eq!(formatter.format(), "1 m");
    let formatter = UnitFormatter::Area {
      unit: Unit::Meters,
      value: 1.0,
    };
    assert_eq!(formatter.format(), "1 m²");
  }

  #[test]
  fn test_convert() {
    let result = Unit::Yards.convert(1.0, &Unit::Feet);
    println!("result: {}", result);
    assert_eq!(result, 3.0);
  }

  #[test]
  fn test_convert_area() {
    let result = UnitUtils::convert_area(1.0, Unit::Meters, Unit::Feet);
    assert_eq!(result, 10.76391);
  }

  #[test]
  fn test_new_unit_value() {
    let unit_value = UnitValue::new(1.0, Unit::Meters, UnitMagnitude::Length);

    assert_eq!(unit_value.length, Some(Length::new::<meter>(1.0)));
    assert_eq!(unit_value.area, None);
    let unit_value = UnitValue::new(1.0, Unit::Meters, UnitMagnitude::Area);
    assert_eq!(unit_value.length, None);
    assert_eq!(unit_value.area, Some(Area::new::<square_meter>(1.0)));
  }
}
