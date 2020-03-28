extern crate libuv;
use libuv::prelude::*;
use libuv::{Buf, FsReq, FsOpenFlags, FsModeFlags};

fn on_read(req: FsReq) {
    match req.result() {
        Err(e) => {
            eprintln!("Read error: {}", e);
            return;
        },
        Ok(0) => {
            if let Err(e) = req.r#loop().fs_close_sync(req.file()) {
                eprintln!("error closing file: {}", e);
            }
        },
    }
}

fn on_open(req: FsReq) {
    if let Err(e) = req.result() {
        eprintln!("error opening file: {}", e);
        return;
    }

    let buf = match Buf::with_capacity(1024) {
        Ok(buf) => buf,
        Err(e) => {
            eprintln!("error allocating a buffer: {}", e);
            return;
        },
    };

    if let Err(e) = req.r#loop().fs_read(req.file(), &[buf], 0, on_read) {
        eprintln!("error starting read: {}", e);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).expect("must pass a path to a file");

    let mut r#loop = Loop::default()?;
    r#loop.fs_open(&path, FsOpenFlags::RDONLY, FsModeFlags::empty(), on_open)?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
