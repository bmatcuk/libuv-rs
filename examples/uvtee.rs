extern crate libuv;
use libuv::prelude::*;
use libuv::{Buf, FsModeFlags, FsOpenFlags, FsReq, ReadonlyBuf};

const STDIN: libuv::File = 0;
const STDOUT: libuv::File = 1;

fn alloc_buffer(handle: Handle, suggested_size: usize) -> buf: Buf {
    Buf::with_capacity(suggested_size)
}

fn read_stdin(stream: StreamHandle, nread: isize, buf: ReadonlyBuf) {
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

    stdin_pipe.read_start(Some(alloc_buffer), Some(read_stdin))?;

    Ok(())
}
