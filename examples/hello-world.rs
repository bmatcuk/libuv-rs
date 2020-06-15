//! Run:
//!
//! ```bash
//! cargo run --example hello-world
//! ```

extern crate libuv;
use libuv::{Loop, RunMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::new()?;

    println!("Now quitting.");
    r#loop.run(RunMode::Default)?;

    // This is not necessary because Loop::drop will call uv_loop_delete, which calls
    // uv_loop_close. Calling uv_loop_close twice will result in an assertion error.
    // r#loop.close()?;

    Ok(())
}
