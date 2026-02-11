#![deny(clippy::all)]

pub mod contour;
pub mod group;
pub mod measurement;
pub mod state;
pub mod utils;
use napi_derive::napi;

/// Add 100 to the input
#[napi]
pub fn plus_100(input: u32) -> u32 {
  input + 100
}

/// Add 200 to the input
#[napi]
pub fn plus_200(input: u32) -> u32 {
  input + 200
}
