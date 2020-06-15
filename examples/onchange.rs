//! Run:
//!
//! ```bash
//! cargo run --example onchange -- file1 file2 ...
//! ```
//!
//! Then "change" one of the watched files with `touch`

extern crate libuv;
use libuv::prelude::*;
use libuv::{FsEvent, FsEventFlags, FsEventHandle};
use std::borrow::Cow;

fn events_to_str(events: FsEvent) -> &'static str {
    if events.contains(FsEvent::RENAME | FsEvent::CHANGE) {
        "changed / renamed"
    } else if events.contains(FsEvent::RENAME) {
        "renamed"
    } else if events.contains(FsEvent::CHANGE) {
        "changed"
    } else {
        ""
    }
}

fn run_command(
    handle: FsEventHandle,
    filename: Option<Cow<str>>,
    events: FsEvent,
    status: libuv::Result<u32>,
) {
    if let Err(e) = status {
        eprintln!(
            "there was an error while watching {}: {}",
            filename.unwrap_or_default(),
            e
        );
        return;
    }

    let path = handle
        .getpath()
        .unwrap_or_else(|_| "(cannot get path)".to_owned());
    eprintln!(
        "Change detected in {}: {} {}",
        path,
        events_to_str(events),
        filename.unwrap_or_default()
    );
}

/// The original implementation runs an arbitrary command when a file changes, but I didn't feel
/// like implementing that...
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        eprintln!("Usage: {} <file1> [file2 ...]", args[0]);
        return Ok(());
    }

    let mut r#loop = Loop::default()?;

    for file in args[1..].iter() {
        println!("Adding watch on {}", file);

        let mut handle = r#loop.fs_event()?;
        handle.start(file, FsEventFlags::RECURSIVE, run_command)?;
    }

    r#loop.run(RunMode::Default)?;

    Ok(())
}
