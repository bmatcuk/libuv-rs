extern crate libuv;
use libuv::prelude::*;
use libuv::{IdleHandle, PrepareHandle};

fn idle_cb(handle: IdleHandle) {
    static mut COUNTER: i32 = 0;

    println!("Idle callback");
    unsafe {
        COUNTER += 1;
        if COUNTER >= 5 {
            println!("Stopping loop");
            handle.get_loop().stop();
        }
    }
}

fn prep_cb(_handle: PrepareHandle) {
    println!("Prep callback");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let mut idle = r#loop.idle()?;
    idle.start(idle_cb)?;

    let mut prep = r#loop.prepare()?;
    prep.start(prep_cb)?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
