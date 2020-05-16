extern crate libuv;
use libuv::prelude::*;
use libuv::{Buf, FsReq, IdleHandle};

fn crunch_away(mut handle: IdleHandle) {
    println!("Computing the meaning of life...");

    // just to avoid overwhelming your terminal emulator
    let _ = handle.stop();
}

fn on_type(req: FsReq, mut idler: IdleHandle, buf: Buf) {
    match req.result() {
        Ok(len) => {
            if len > 0 {
                if let Ok(s) = buf.readonly().to_str(len) {
                    println!("Typed {}", s);
                }
                let _ = idler.start(crunch_away);
                let _ = req
                    .r#loop()
                    .fs_read(0, &[buf], -1, move |req: FsReq| on_type(req, idler, buf));
            }
        }
        Err(e) => {
            eprintln!("Error opening file: {}", e);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let mut idler = r#loop.idle()?;
    idler.start(crunch_away)?;

    let buf = Buf::with_capacity(1024)?;
    r#loop.fs_read(0, &[buf], -1, move |req: FsReq| on_type(req, idler, buf))?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
