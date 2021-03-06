//! Run:
//!
//! ```bash
//! cargo run --example detach
//! ```
//!
//! It will spawn a new process, print the PID, and detach. You can check that the process was
//! spawned by using `ps ax | grep PID`. It will run for 100 seconds, after which you should see
//! the process disappear by re-running `ps`.

extern crate libuv;
use libuv::prelude::*;
use libuv::{ProcessFlags, ProcessOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let mut options = ProcessOptions::new(&["sleep", "100"]);
    options.flags = ProcessFlags::DETACHED;

    let mut process = r#loop.spawn_process(options)?;
    println!("Launched sleep with PID {}", process.pid());
    process.unref();

    r#loop.run(RunMode::Default)?;

    Ok(())
}
