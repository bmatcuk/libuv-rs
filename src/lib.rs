#[macro_use]
extern crate libuv_sys2 as uv;

pub mod error;
pub use error::*;
pub use error::Error::*;

pub type Result<T> = std::result::Result<T, error::Error>;
