extern crate libuv;
use libuv::prelude::*;
use libuv::{exepath, ProcessHandle, ProcessOptions, StdioContainer, StdioFlags, StdioType};
use std::path::PathBuf;

fn on_exit(mut handle: ProcessHandle, exit_status: i64, term_signal: i32) {
    println!(
        "Process exited with status {}, signal {}",
        exit_status, term_signal
    );
    handle.close(());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let mut path: PathBuf = exepath()?.into();
    if cfg!(windows) {
        path.set_file_name("proc-streams-test.exe");
    } else {
        path.set_file_name("proc-streams-test");
    }

    let child_stdio = [
        Default::default(),
        Default::default(),
        StdioContainer {
            flags: StdioFlags::INHERIT_FD,
            data: StdioType::Fd(2),
        },
    ];

    let path = path.to_string_lossy().into_owned();
    let args: [&str; 1] = [&path];
    let mut options = ProcessOptions::new(&args);
    options.exit_cb = on_exit.into();
    options.stdio = &child_stdio;

    r#loop.spawn_process(options)?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
