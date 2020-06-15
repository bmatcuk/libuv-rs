//! Run:
//!
//! ```bash
//! cargo run --example progress
//! ```

extern crate libuv;
use libuv::prelude::*;
use libuv::AsyncHandle;
use std::cell::Cell;
use std::rc::Rc;

const SIZE: usize = 10240;

fn fake_download(notify: &mut AsyncHandle, progress: &Rc<Cell<f32>>) {
    let mut downloaded = 0usize;
    while downloaded < SIZE {
        progress.replace((downloaded as f32) * 100.0 / (SIZE as f32));
        if let Err(e) = notify.send() {
            eprintln!("Failed to send async notification {}", e);
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
        downloaded += (rand::random::<usize>() + 200) % 1000;
    }
}

fn after(notify: &mut AsyncHandle) {
    println!("Download complete");
    let _ = notify.close(());
}

fn print_progress(progress: &Rc<Cell<f32>>) {
    println!("Downloaded {}%", progress.get());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let progress = Rc::new(Cell::new(0f32));
    let progress2 = progress.clone();
    let mut notify = r#loop.r#async(move |_| print_progress(&progress))?;
    r#loop.queue_work(
        move |_| fake_download(&mut notify, &progress2),
        move |_, _| after(&mut notify),
    )?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
