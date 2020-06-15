//! This is part of the multi-echo-server example. See multi-echo-server.rs for documentation

extern crate libuv;
use libuv::prelude::*;
use libuv::{getpid, Buf, HandleType, PipeHandle, ReadonlyBuf};
use std::convert::TryInto;

fn alloc_buffer(_handle: Handle, suggested_size: usize) -> Option<Buf> {
    Buf::with_capacity(suggested_size).ok()
}

fn echo_write(status: libuv::Result<u32>, mut buf: ReadonlyBuf) {
    if let Err(e) = status {
        eprintln!("Write error {}", e);
    }
    buf.dealloc();
}

fn echo_read(mut client: StreamHandle, nread: libuv::Result<usize>, buf: ReadonlyBuf) {
    match nread {
        Ok(len) => {
            if len > 0 {
                if let Err(e) = client.write(&[buf], move |_, s| echo_write(s, buf)) {
                    eprintln!("Error echoing {}", e);
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

fn on_new_connection(mut stream: StreamHandle, nread: libuv::Result<usize>, mut buf: ReadonlyBuf) {
    match nread {
        Ok(_) => {
            let mut pipe: PipeHandle = stream.try_into().unwrap();
            if pipe.pending_count() == 0 {
                eprintln!("No pending count");
                return;
            }

            if pipe.pending_type() != HandleType::TCP {
                eprintln!("Wrong type: {}", pipe.pending_type());
                return;
            }

            if let Ok(mut client) = pipe.get_loop().tcp() {
                match pipe.accept(&mut client.to_stream()) {
                    Ok(_) => {
                        if let Ok(file) = pipe.get_fileno() {
                            println!("Worker {}: Accepted fd {}", getpid(), file);
                        }
                        let _ = client.read_start(alloc_buffer, echo_read);
                    }
                    Err(_) => {
                        client.close(());
                    }
                }
            }
        }
        Err(e) => {
            if e != libuv::Error::EOF {
                eprintln!("Read error {}", e);
            }
            stream.close(());
        }
    }

    buf.dealloc();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let mut queue = r#loop.pipe(true)?;
    queue.open(0)?;
    queue.read_start(alloc_buffer, on_new_connection)?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
