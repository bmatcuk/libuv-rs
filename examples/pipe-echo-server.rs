//! Run:
//!
//! ```bash
//! cargo run --example pipe-echo-server
//! ```
//!
//! Then connect using:
//!
//! ```bash
//! socat - /tmp/echo.sock
//! ```
//!
//! Anything you type will be echoed back.

extern crate libuv;
use libuv::prelude::*;
use libuv::{Buf, ReadonlyBuf, SignalHandle};
use libuv_sys2::SIGINT;

#[cfg(windows)]
const PIPENAME: &str = r"\\?\pipe\echo.sock";

#[cfg(not(windows))]
const PIPENAME: &str = "/tmp/echo.sock";

fn alloc_buffer(_: Handle, suggested_size: usize) -> Option<Buf> {
    Buf::with_capacity(suggested_size).ok()
}

fn echo_write(mut buf: ReadonlyBuf, status: libuv::Result<u32>) {
    if let Err(e) = status {
        eprintln!("Write error {}", e);
    }
    buf.dealloc();
}

fn echo_read(mut client: StreamHandle, nread: libuv::Result<usize>, buf: ReadonlyBuf) {
    match nread {
        Ok(len) => {
            if len > 0 {
                if let Err(e) = client.write(&[buf], move |_, s| echo_write(buf, s)) {
                    eprintln!("Error echoing to pipe: {}", e);
                }
            }
        }
        Err(e) => {
            if e != libuv::Error::EOF {
                eprintln!("Read error {}", e);
            }
            client.close(());
        }
    }
}

fn on_new_connection(mut server: StreamHandle, status: libuv::Result<u32>) {
    if let Err(e) = status {
        eprintln!("New connection error: {}", e);
        return;
    }

    if let Ok(mut client) = server.get_loop().pipe(false) {
        match server.accept(&mut client.to_stream()) {
            Ok(_) => {
                let _ = client.read_start(alloc_buffer, echo_read);
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
                client.close(());
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let mut server = r#loop.pipe(false)?;
    server.bind(PIPENAME)?;
    server.listen(128, on_new_connection)?;

    let mut sig = r#loop.signal()?;
    sig.start(
        move |mut sig: SignalHandle, _| {
            let _ = sig.get_loop().fs_unlink_sync(PIPENAME);
            let _ = sig.stop();
            server.close(());
        },
        SIGINT as _,
    )?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
