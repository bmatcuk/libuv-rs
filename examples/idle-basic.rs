extern crate libuv;
use libuv::{Loop, RunMode, IdleHandle, HandleTrait, Handle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::new()?;

    let mut count = 0u64;
    let mut idle = r#loop.idle()?;
    idle.start(Some(move |mut handle: IdleHandle| {
        count += 1;

        if count >= 10_000_000 {
            handle.stop().unwrap();

            // Because Loop::drop() calls uv_loop_delete, we need to make sure our idle handle is
            // closed, in addition to stopped, before main reaches the end.
            handle.close(None::<fn(Handle)>);
        }
    }))?;

    println!("Idling...");
    r#loop.run(RunMode::Default)?;

    Ok(())
}
