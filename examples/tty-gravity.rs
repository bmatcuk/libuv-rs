extern crate libuv;
use libuv::prelude::*;
use libuv::{Buf, TtyHandle, TtyMode};

const STDOUT: libuv::File = 1;
const MESSAGE: &str = "  Hello TTY  ";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let mut tty = r#loop.tty(STDOUT)?;
    tty.set_mode(TtyMode::Normal)?;

    match tty.get_winsize() {
        Ok((width, height)) => {
            println!("Width {}, Height {}", width, height);

            let mut tick = r#loop.timer()?;
            let mut pos = 0;
            tick.start(200, 200, move |_| {
                if let Ok(buf) = Buf::new(&format!(
                    "\x1b[2J\x1b[H\x1b[{}B\x1b[{}C\x1b[42;37m{}",
                    pos,
                    (width - MESSAGE.len() as i32) / 2,
                    MESSAGE
                )) {
                    let _ = tty.write(&[buf], ());
                }

                pos += 1;
                if pos > height {
                    let _ = TtyHandle::reset_mode();
                    let _ = tick.stop();
                }
            })?;

            r#loop.run(RunMode::Default)?;
        }
        Err(e) => {
            eprintln!("Could not get TTY information: {}", e);
        }
    }

    Ok(())
}
