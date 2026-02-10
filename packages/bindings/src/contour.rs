use crate::state::TakeoffStateHandler;
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
    *self._surface_mesh.lock().unwrap() = Some(surface_mesh);
    Ok(())
  }

  #[napi(constructor)]
  pub fn with_default_state(contour: ContourInput) -> Self {
    Self::new(contour, Arc::new(TakeoffStateHandler::default()))
  }

  /// Get the surface points for the contour.
  #[napi]
  pub fn get_surface_points(&self) -> Option<Vec<Point3D>> {
    if let Some(surface_mesh) = self._surface_mesh.lock().unwrap().as_ref() {
      return Some(surface_mesh.vertices.clone());
    };
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
    let mesh = self._surface_mesh.lock().ok()?;
    let mesh = mesh.as_ref()?;
    let reference = ReferenceSurface::from(reference);
    Some(mesh.volume_against(&reference, cell_size))
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
    // let surface_mesh = self._surface_mesh.lock().unwrap().as_ref().unwrap();
    let bounding_box = self.contour.bounding_box();
    if let Some(bounding_box) = bounding_box {
      let (min_x, min_y) = bounding_box.0;
      let (max_x, max_y) = bounding_box.1;
      let mut data: Vec<Point3D> = Vec::new();
      println!(
        "min_x: {}, min_y: {}, max_x: {}, max_y: {}",
        min_x, min_y, max_x, max_y
      );
      let x_start = min_x as i32;
      let x_end = max_x as i32;
      let y_start = min_y as i32;
      let y_end = max_y as i32;
      for x in (x_start..x_end).step_by(step as usize) {
        for y in (y_start..y_end).step_by(step as usize) {
          let z = self
            ._surface_mesh
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .z_at(x as f64, y as f64);
          if let Some(z) = z {
            data.push(Point3D::new(x as f64, y as f64, z));
          }
        }
      }
      return Some(data);
    }
    None
  }
}
