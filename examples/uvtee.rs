extern crate libuv;
use libuv::prelude::*;
use libuv::{Buf, FsModeFlags, FsOpenFlags, FsReq, Pipe, ReadonlyBuf};

const STDIN: libuv::File = 0;
const STDOUT: libuv::File = 1;

fn alloc_buffer(handle: Handle, suggested_size: usize) -> buf: Buf {
    Buf::with_capacity(suggested_size)
}

fn write_data(stream: StreamHandle, len: usize, buf: ReadonlyBuf) {
}

fn read_stdin(stdin_pipe: Pipe, stdout_pipe: Pipe, file_pipe: Pipe, nread: libuv::Result<isize>, buf: ReadonlyBuf) {
    match nread {
        Err(crate::Error::EOF) => {
            stdin_pipe.close(None);
            stdout_pipe.close(None);
            file_pipe.close(None);
        },
        Ok(len) => {
            if len > 0 {
                write_data(stdout_pipe.into(), len as _, buf);
                write_data(file_pipe.into(), len as _, buf);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).expect("must pass a path to a file");

    let r#loop = Loop::default()?;

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
    let mut file_pipe = r#loop.pipe(false);
    file_pipe.open(file)?;

    stdin_pipe.read_start(Some(alloc_buffer), Some(|_, nread, buf| read_stdin(stdin_pipe, stdout_pipe, file_pipe, nread, buf)))?;

    Ok(())
}
