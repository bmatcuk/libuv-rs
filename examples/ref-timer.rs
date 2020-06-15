//! Run:
//!
//! ```bash
//! cargo run --example ref-timer
//! ```

extern crate libuv;
use libuv::prelude::*;
use libuv::TimerHandle;

fn gc(_handle: TimerHandle) {
    println!("Freeing unused objects");
}

fn fake_job(_handle: TimerHandle) {
    println!("Fake job done");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    // by unref'ing this handle, the loop will end when fake_job_timer ends even though this handle
    // is still running
    let mut timer = r#loop.timer()?;
    timer.unref();
    timer.start(0, 2000, gc)?;

    let mut fake_job_timer = r#loop.timer()?;
    fake_job_timer.start(9000, 0, fake_job)?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
