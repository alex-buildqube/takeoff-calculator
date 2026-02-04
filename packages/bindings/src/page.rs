use napi_derive::napi;
use serde::{Deserialize, Serialize};

#[napi(object)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageViewport {
  pub width: f64,
  pub height: f64,
}

#[napi(object)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page {
  pub id: String,
  pub name: Option<String>,
  pub width: Option<f64>,
  pub height: Option<f64>,
  pub viewport: Option<PageViewport>,
}
