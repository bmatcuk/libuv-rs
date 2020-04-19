extern crate libuv;
use libuv::prelude::*;
use libuv::{AsyncHandle};

const SIZE: usize = 10240;

fn fake_download(notify: &mut AsyncHandle, progress: &mut f32) {
    let mut downloaded = 0usize;
    while downloaded < SIZE {
        *progress = (downloaded as f32) * 100.0 / (SIZE as f32);
        if let Err(e) = notify.send() {
            eprintln!("Failed to send async notification {}", e);
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
        downloaded += (rand::random::<usize>() + 200) % 1000;
    }
}

fn after(notify: &mut AsyncHandle) {
    println!("Download complete");
    let _ = notify.close(None::<fn(_)>);
}

fn print_progress(progress: &f32) {
    println!("Downloaded {}%", progress);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let mut progress = 0f32;
    let mut notify = r#loop.r#async(Some(|_| print_progress(&progress)))?;
    r#loop.queue_work(Some(|_| fake_download(&mut notify, &mut progress)), Some(|_, _| after(&mut notify)))?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
