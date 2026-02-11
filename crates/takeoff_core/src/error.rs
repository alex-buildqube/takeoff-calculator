use serde::{Deserialize, Serialize};
use thiserror::Error;

use napi::{Error as NapiError, Status};

/// Error type for takeoff_core operations.
///
/// All invalid-input cases return a variant of `TakeoffError`; no silent fallbacks
/// that produce incorrect numbers.
#[derive(Debug, Clone, PartialEq, Error, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TakeoffError {
  /// Empty or invalid geometry was provided.
  ///
  /// This error is returned when:
  /// - A polygon has fewer than 3 points
  /// - A polyline has fewer than 2 points
  /// - A rectangle has invalid or identical corner points
  /// - Degenerate geometry (e.g., all collinear points) that cannot form a valid shape
  #[error("empty or invalid geometry: {message}")]
  EmptyGeometry {
    /// Human-readable message describing why the geometry is invalid
    message: String,
  },

  /// Invalid scale ratio was provided.
  ///
  /// This error is returned when:
  /// - Scale ratio is zero (division by zero)
  /// - Scale ratio is negative
  /// - Real distance is zero or negative
  /// - Pixel distance is zero or negative
  #[error("invalid scale: {message}")]
  InvalidScale {
    /// Human-readable message describing why the scale is invalid
    message: String,
  },

  /// Unknown or unsupported unit was provided.
  ///
  /// This error is returned when:
  /// - A unit string cannot be parsed or is not recognized
  /// - A unit is requested that is not supported for the operation (e.g., requesting area in a length-only context)
  #[error("unknown or unsupported unit: {unit}")]
  UnknownUnit {
    /// The unit string that was unknown or unsupported
    unit: String,
  },

  // Contour Errors
  /// Too few points for triangulation (need at least 3).
  #[error("too few points for triangulation: {count} (need at least 3)")]
  SurfaceMeshTooFewPoints { count: usize },
  /// All points are collinear; Delaunay triangulation produces no triangles.
  #[error("all points are collinear; cannot create triangulated surface")]
  SurfaceMeshCollinearPoints,

  // System Errors
  /// A mutex or lock was poisoned (a thread panicked while holding the lock).
  ///
  /// This error is returned when:
  /// - A mutex lock operation fails because another thread panicked while holding the lock
  /// - Internal synchronization state is corrupted
  ///
  /// This typically indicates a programming error or unexpected panic in concurrent code.
  #[error("mutex lock poisoned: {resource}")]
  PoisonError {
    /// The name of the resource that was locked (e.g., "area", "scale", "measurement")
    resource: String,
  },

  // Catchall Error
  #[error("an unknown error occurred: {message}")]
  UnknownError { message: String },
}

impl TakeoffError {
  /// Create an `EmptyGeometry` error with a message.
  pub fn empty_geometry(message: impl Into<String>) -> Self {
    Self::EmptyGeometry {
      message: message.into(),
    }
  }

  /// Create an `InvalidScale` error with a message.
  pub fn invalid_scale(message: impl Into<String>) -> Self {
    Self::InvalidScale {
      message: message.into(),
    }
  }

  /// Create an `UnknownUnit` error with the unit string.
  pub fn unknown_unit(unit: impl Into<String>) -> Self {
    Self::UnknownUnit { unit: unit.into() }
  }

  /// Create a `PoisonError` error for a poisoned mutex lock.
  pub fn poison_error(resource: impl Into<String>) -> Self {
    Self::PoisonError {
      resource: resource.into(),
    }
  }
}

impl From<TakeoffError> for NapiError {
  fn from(error: TakeoffError) -> Self {
    match error {
      TakeoffError::EmptyGeometry { message } => NapiError::new(Status::InvalidArg, message),
      TakeoffError::InvalidScale { message } => NapiError::new(Status::InvalidArg, message),
      TakeoffError::UnknownUnit { unit } => NapiError::new(Status::InvalidArg, unit),
      TakeoffError::SurfaceMeshTooFewPoints { .. } => {
        NapiError::new(Status::InvalidArg, error.to_string())
      }
      TakeoffError::SurfaceMeshCollinearPoints => {
        NapiError::new(Status::InvalidArg, error.to_string())
      }
      TakeoffError::PoisonError { resource } => NapiError::new(
        Status::GenericFailure,
        format!("mutex lock poisoned: {}", resource),
      ),
      TakeoffError::UnknownError { message } => NapiError::new(Status::InvalidArg, message),
    }
  }
}

impl From<NapiError> for TakeoffError {
  fn from(error: NapiError) -> Self {
    TakeoffError::UnknownError {
      message: error.to_string(),
    }
  }
}

impl<T> From<std::sync::PoisonError<T>> for TakeoffError {
  fn from(_error: std::sync::PoisonError<T>) -> Self {
    TakeoffError::PoisonError {
      resource: "unknown".to_string(),
    }
  }
}

pub type TakeoffResult<T> = Result<T, TakeoffError>;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_error_serialization() {
    let err = TakeoffError::empty_geometry("polygon has fewer than 3 points");
    let serialized = serde_json::to_string(&err).unwrap();
    assert!(serialized.contains("emptyGeometry"));
    assert!(serialized.contains("polygon has fewer than 3 points"));

    let err = TakeoffError::invalid_scale("scale ratio cannot be zero");
    let serialized = serde_json::to_string(&err).unwrap();
    assert!(serialized.contains("invalidScale"));
    assert!(serialized.contains("scale ratio cannot be zero"));

    let err = TakeoffError::unknown_unit("kilometers");
    let serialized = serde_json::to_string(&err).unwrap();
    assert!(serialized.contains("unknownUnit"));
    assert!(serialized.contains("kilometers"));

    let err = TakeoffError::poison_error("scale");
    let serialized = serde_json::to_string(&err).unwrap();
    assert!(serialized.contains("poisonError"));
    assert!(serialized.contains("scale"));
  }

  #[test]
  fn test_error_deserialization() {
    let json = r#"{"type":"emptyGeometry","message":"polygon has fewer than 3 points"}"#;
    let err: TakeoffError = serde_json::from_str(json).unwrap();
    assert!(matches!(err, TakeoffError::EmptyGeometry { .. }));

    let json = r#"{"type":"invalidScale","message":"scale ratio cannot be zero"}"#;
    let err: TakeoffError = serde_json::from_str(json).unwrap();
    assert!(matches!(err, TakeoffError::InvalidScale { .. }));

    let json = r#"{"type":"unknownUnit","unit":"kilometers"}"#;
    let err: TakeoffError = serde_json::from_str(json).unwrap();
    assert!(matches!(err, TakeoffError::UnknownUnit { .. }));

    let json = r#"{"type":"poisonError","resource":"measurement"}"#;
    let err: TakeoffError = serde_json::from_str(json).unwrap();
    assert!(matches!(err, TakeoffError::PoisonError { resource } if resource == "measurement"));
  }

  #[test]
  fn test_error_display() {
    let err = TakeoffError::empty_geometry("polygon has fewer than 3 points");
    let display = format!("{}", err);
    assert!(display.contains("empty or invalid geometry"));
    assert!(display.contains("polygon has fewer than 3 points"));

    let err = TakeoffError::invalid_scale("scale ratio cannot be zero");
    let display = format!("{}", err);
    assert!(display.contains("invalid scale"));
    assert!(display.contains("scale ratio cannot be zero"));

    let err = TakeoffError::unknown_unit("kilometers");
    let display = format!("{}", err);
    assert!(display.contains("unknown or unsupported unit"));
    assert!(display.contains("kilometers"));

    let err = TakeoffError::SurfaceMeshTooFewPoints { count: 2 };
    let display = format!("{}", err);
    assert!(display.contains("too few points for triangulation"));
    assert!(display.contains("2"));

    let err = TakeoffError::SurfaceMeshCollinearPoints;

    let display = format!("{}", err);

    assert!(display.contains("all points are collinear"));
    assert!(display.contains("cannot create triangulated surface"));

    let err = TakeoffError::poison_error("scale");
    let display = format!("{}", err);
    assert!(display.contains("mutex lock poisoned"));
    assert!(display.contains("scale"));
  }
}
