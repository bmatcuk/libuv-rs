extern crate libuv;
use libuv::prelude::*;
use libuv::{Buf, FsModeFlags, FsOpenFlags, PipeHandle, ReadonlyBuf};

const STDIN: libuv::File = 0;
const STDOUT: libuv::File = 1;

fn alloc_buffer(_handle: Handle, suggested_size: usize) -> Option<Buf> {
    match Buf::with_capacity(suggested_size) {
        Ok(b) => Some(b),
        Err(e) => {
            eprintln!("error allocating buffer: {}", e);
            None
        }
    }
}

fn write_data(mut stream: StreamHandle, len: usize, buf: &ReadonlyBuf) -> libuv::Result<()> {
    let mut buf = Buf::new_from(buf, Some(len))?;
    stream.write(
        &[buf],
        Some(move |_, status| {
            if let Err(e) = status {
                eprintln!("error writing data: {}", e);
            }
            buf.destroy();
        }),
    )?;
    Ok(())
}

fn read_stdin(
    mut stdin_pipe: StreamHandle,
    stdout_pipe: &mut PipeHandle,
    file_pipe: &mut PipeHandle,
    nread: libuv::Result<isize>,
    mut buf: ReadonlyBuf,
) {
    match nread {
        Err(e) => {
            if e != libuv::Error::EOF {
                eprintln!("error reading stdin: {}", e);
            }
            stdin_pipe.close(None::<fn(Handle)>);
            stdout_pipe.close(None::<fn(Handle)>);
            file_pipe.close(None::<fn(Handle)>);
        }
        Ok(len) => {
            if len > 0 {
                if let Err(e) = write_data(stdout_pipe.to_stream(), len as _, &buf) {
                    eprintln!("error preparing to writing to stdout: {}", e);
                }

                if let Err(e) = write_data(file_pipe.to_stream(), len as _, &buf) {
                    eprintln!("error preparing to writing to file: {}", e);
                }
            }
        }
    }

    // free memory in the ReadonlyBuf
    buf.dealloc();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).expect("must pass a path to a file");

    let mut r#loop = Loop::default()?;

    let mut stdin_pipe = r#loop.pipe(false)?;
    stdin_pipe.open(STDIN)?;

    let mut stdout_pipe = r#loop.pipe(false)?;
    stdout_pipe.open(STDOUT)?;

    let file = r#loop.fs_open_sync(
        &path,
        FsOpenFlags::CREAT | FsOpenFlags::RDWR,
        FsModeFlags::OWNER_READ
            | FsModeFlags::OWNER_WRITE
            | FsModeFlags::GROUP_READ
            | FsModeFlags::OTHERS_READ,
    )?;
    let mut file_pipe = r#loop.pipe(false)?;
    file_pipe.open(file)?;

    stdin_pipe.read_start(
        Some(alloc_buffer),
        Some(move |stream, nread, buf| {
            read_stdin(stream, &mut stdout_pipe, &mut file_pipe, nread, buf)
        }),
    )?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
