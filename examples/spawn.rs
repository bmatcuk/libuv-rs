extern crate libuv;
use libuv::prelude::*;
use libuv::{ProcessHandle, ProcessOptions};

fn on_exit(mut handle: ProcessHandle, exit_status: i64, term_signal: i32) {
    println!(
        "Process exited with status {}, signal {}",
        exit_status, term_signal
    );
    handle.close(());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let mut options = ProcessOptions::new(&["mkdir", "test-dir"]);
    options.exit_cb = on_exit.into();

    let process = r#loop.spawn_process(options)?;
    println!("Launched process with ID {}", process.pid());

    r#loop.run(RunMode::Default)?;

    Ok(())
}
