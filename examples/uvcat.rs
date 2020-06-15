//! Run:
//!
//! ```bash
//! cargo run --example uvcat -- file
//! ```

extern crate libuv;
use libuv::prelude::*;
use libuv::{Buf, FsModeFlags, FsOpenFlags, FsReq};

const STDOUT: libuv::File = 1;

fn on_write(req: FsReq, file: libuv::File, mut buf: Buf) {
    if let Err(e) = req.result() {
        eprintln!("Write error: {}", e);
        buf.destroy();
    } else if let Err(e) = req
        .r#loop()
        .fs_read(file, &[buf], -1, move |req| on_read(file, req, buf))
    {
        eprintln!("error continuing read: {}", e);
    }
}

fn on_read(fd: libuv::File, req: FsReq, mut buf: Buf) {
    match req.result() {
        Err(e) => {
            eprintln!("Read error: {}", e);
            buf.destroy();
            return;
        }
        Ok(0) => {
            buf.destroy();
            if let Err(e) = req.r#loop().fs_close_sync(fd) {
                eprintln!("error closing file: {}", e);
            }
        }
        Ok(len) => {
            if let Err(e) = buf.resize(len as _) {
                eprintln!("error resizing buffer: {}", e);
                buf.destroy();
            }
            if let Err(e) = req
                .r#loop()
                .fs_write(STDOUT, &[buf], -1, move |req| on_write(req, fd, buf))
            {
                eprintln!("error starting write: {}", e);
                buf.destroy();
            }
        }
    }
}

fn on_open(req: FsReq) {
    match req.result() {
        Ok(fd) => {
            let mut buf = match Buf::with_capacity(1024) {
                Ok(buf) => buf,
                Err(e) => {
                    eprintln!("error allocating a buffer: {}", e);
                    return;
                }
            };

            let fd = fd as libuv::File;
            if let Err(e) = req
                .r#loop()
                .fs_read(fd, &[buf], -1, move |req| on_read(fd, req, buf))
            {
                eprintln!("error starting read: {}", e);
                buf.destroy();
            }
        }
        Err(e) => eprintln!("error opening file: {}", e),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).expect("must pass a path to a file");

    let mut r#loop = Loop::default()?;
    r#loop.fs_open(&path, FsOpenFlags::RDONLY, FsModeFlags::empty(), on_open)?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
