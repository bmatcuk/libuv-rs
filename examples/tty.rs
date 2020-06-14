extern crate libuv;
use libuv::prelude::*;
use libuv::{guess_handle, Buf, HandleType, TtyHandle, TtyMode};

const STDOUT: libuv::File = 1;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let mut tty = r#loop.tty(STDOUT)?;
    tty.set_mode(TtyMode::Normal)?;
    if guess_handle(STDOUT) == HandleType::TTY {
        let buf = Buf::new("\x1b[41;37m")?;
        tty.write(&[buf], ())?;
    }

    let buf = Buf::new("Hello TTY\n")?;
    tty.write(&[buf], ())?;
    TtyHandle::reset_mode()?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
