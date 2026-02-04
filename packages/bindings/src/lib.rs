#![deny(clippy::all)]
pub mod coords;
pub mod group;
pub mod measurement;
pub mod page;
pub mod scale;
pub mod state;
pub mod unit;
use napi_derive::napi;

#[napi]
pub fn plus_100(input: u32) -> u32 {
  input + 100
}
