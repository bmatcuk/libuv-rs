extern crate libuv;
use libuv::prelude::*;
use libuv::{Buf, FsReq, FsOpenFlags, FsModeFlags};

fn on_write(req: FsReq, file: libuv::File, mut buf: Buf) {
    if let Err(e) = req.result() {
        eprintln!("Write error: {}", e);
        buf.destroy();
    } else if let Err(e) = req.r#loop().fs_read(file, &[buf], 0, move |req| on_read(req. buf)) {
        eprintln!("error continuing read: {}", e);
    }
}

fn on_read(req: FsReq, mut buf: Buf) {
    match req.result() {
        Err(e) => {
            eprintln!("Read error: {}", e);
            buf.destroy();
            return;
        },
        Ok(0) => {
            buf.destroy();
            if let Err(e) = req.r#loop().fs_close_sync(req.file()) {
                eprintln!("error closing file: {}", e);
            }
        },
        Ok(len) => {
            let file = req.file();
            buf.truncate(len as _);
            if let Err(e) = req.r#loop().fs_write(1 as libuv::File, &[buf], 0, move |req| on_write(req, file, buf)) {
                eprintln!("error starting write: {}", e);
                buf.destroy();
            }
        }
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

    if let Err(e) = req.r#loop().fs_read(req.file(), &[buf], 0, move |req| on_read(req, buf)) {
        eprintln!("error starting read: {}", e);
        buf.destroy();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).expect("must pass a path to a file");

    let mut r#loop = Loop::default()?;
    r#loop.fs_open(&path, FsOpenFlags::RDONLY, FsModeFlags::empty(), on_open)?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
