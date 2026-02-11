use crate::state::TakeoffStateHandler;
use crate::utils::lock_mutex;
use anyhow::Result;
use napi_derive::napi;
use std::sync::{Arc, Mutex};
use takeoff_core::contour::{ContourInput, SurfaceMesh};
use takeoff_core::coords::Point3D;
use takeoff_core::volume::{ReferenceSurface, ReferenceSurfaceInput, VolumetricResult};

#[napi]
#[derive(Debug, Clone)]
pub struct ContourWrapper {
  contour: ContourInput,
  _state: Arc<TakeoffStateHandler>,
  _surface_mesh: Arc<Mutex<Option<SurfaceMesh>>>,
}

#[napi]
impl ContourWrapper {
  pub fn new(contour: ContourInput, state: Arc<TakeoffStateHandler>) -> Self {
    let res = Self {
      contour,
      _state: state,
      _surface_mesh: Arc::new(Mutex::new(None)),
    };

    let _ = res.initialize_surface_mesh();
    res
  }

  fn initialize_surface_mesh(&self) -> Result<()> {
    let surface_mesh = self.contour.to_surface_mesh()?;
    *lock_mutex(self._surface_mesh.lock(), "surface_mesh")? = Some(surface_mesh);
    Ok(())
  }

  #[napi(constructor)]
  pub fn with_default_state(contour: ContourInput) -> Self {
    Self::new(contour, Arc::new(TakeoffStateHandler::default()))
  }

  /// Get the surface points for the contour.
  ///
  /// Returns `None` if the surface mesh is not available or if the mutex is poisoned.
  #[napi]
  pub fn get_surface_points(&self) -> Option<Vec<Point3D>> {
    if let Ok(surface_mesh) = lock_mutex(self._surface_mesh.lock(), "surface_mesh") {
      if let Some(surface_mesh) = surface_mesh.as_ref() {
        return Some(surface_mesh.vertices.clone());
      }
    }
    None
  }

  /// Compute cut/fill volume against a reference surface.
  /// Returns None if the surface mesh is not available (e.g. contour conversion failed).
  #[napi]
  pub fn volume_against(
    &self,
    reference: ReferenceSurfaceInput,
    cell_size: Option<f64>,
  ) -> Option<VolumetricResult> {
    let mesh = lock_mutex(self._surface_mesh.lock(), "surface_mesh").ok()?;
    let mesh = mesh.as_ref()?;
    let reference = ReferenceSurface::from(reference);
    Some(mesh.volume_against(&reference, cell_size))
  }

  /// Get the z value at a given x, y coordinate.
  /// Returns None if the surface mesh is not available (e.g. contour conversion failed).
  #[napi]
  pub fn get_z_at(&self, x: f64, y: f64) -> Option<f64> {
    let mesh = lock_mutex(self._surface_mesh.lock(), "surface_mesh").ok()?;
    let mesh = mesh.as_ref()?;
    mesh.z_at(x, y)
  }

  /// Get scatter data for the contour.
  /// Returns None if the surface mesh is not available (e.g. contour conversion failed).
  ///
  /// # Arguments
  ///
  /// * `step` - The step size for the scatter data.
  ///
  /// # Returns
  ///
  /// * `Vec<Point3D>` - The scatter data.
  /// * `None` - If the surface mesh is not available (e.g. contour conversion failed).
  #[napi]
  pub fn get_scatter_data(&self, step: i32) -> Option<Vec<Point3D>> {
    if step <= 0 {
      return None;
    }
    let step = step as usize;

    let bounding_box = self.contour.bounding_box()?;
    let mesh_guard = lock_mutex(self._surface_mesh.lock(), "surface_mesh").ok()?;
    let surface_mesh = mesh_guard.as_ref()?;

    let (min_x, min_y) = bounding_box.0;
    let (max_x, max_y) = bounding_box.1;
    let mut data: Vec<Point3D> = Vec::new();

    let x_start = min_x.floor() as i32;
    let x_end = max_x.ceil() as i32;
    let y_start = min_y.floor() as i32;
    let y_end = max_y.ceil() as i32;

    println!(
      "x_start: {}, x_end: {}, y_start: {}, y_end: {}",
      x_start, x_end, y_start, y_end
    );

    for x in (x_start..=x_end).step_by(step) {
      for y in (y_start..=y_end).step_by(step) {
        if let Some(z) = surface_mesh.z_at(x as f64, y as f64) {
          data.push(Point3D::new(x as f64, y as f64, z));
        }
      }
    }
    Some(data)
  }
}

#[cfg(test)]
mod tests {
  use takeoff_core::contour::ContourLineInput;
  use takeoff_core::coords::Point;

  use super::*;

  #[test]
  fn test_get_scatter_data() {
    let contour = ContourWrapper::with_default_state(ContourInput {
      id: "1".to_string(),
      name: None,
      page_id: "1".to_string(),
      lines: vec![ContourLineInput {
        elevation: 0.0,
        points: vec![
          Point::new(0.0, 0.0),
          Point::new(100.0, 0.0),
          Point::new(100.0, 100.0),
          Point::new(0.0, 100.0),
        ],
      }],
      points_of_interest: vec![],
    });
    let scatter_data = contour.get_scatter_data(10);
    assert!(scatter_data.is_some());
    assert_eq!(scatter_data.unwrap().len(), 121);
  }
}
