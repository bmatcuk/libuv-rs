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
