extern crate libuv;
use libuv::prelude::*;
use libuv::IdleHandle;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let mut count = 0u64;
    let mut idle = r#loop.idle()?;
    idle.start(move |mut handle: IdleHandle| {
        count += 1;

        if count >= 10_000_000 {
            handle.stop().unwrap();
        }
    })?;

    println!("Idling...");
    r#loop.run(RunMode::Default)?;

    Ok(())
}
