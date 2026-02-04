
use napi_derive::napi;
use serde::{Deserialize, Serialize};

use crate::group::Group;
use crate::measurement::Measurement;
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
