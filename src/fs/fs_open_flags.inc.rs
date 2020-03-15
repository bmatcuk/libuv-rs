#[allow(non_camel_case_types)]
bitflags! {
    pub struct FsOpenFlags: i32 {
        const APPEND = uv::UV_FS_O_APPEND as _;
        const CREAT = uv::UV_FS_O_CREAT as _;
        const DIRECT = uv::UV_FS_O_DIRECT as _;
        const DIRECTORY = uv::UV_FS_O_DIRECTORY as _;
        const DSYNC = uv::UV_FS_O_DSYNC as _;
        const EXCL = uv::UV_FS_O_EXCL as _;
        const EXLOCK = uv::UV_FS_O_EXLOCK as _;
        const FILEMAP = uv::UV_FS_O_FILEMAP as _;
        const NOATIME = uv::UV_FS_O_NOATIME as _;
        const NOCTTY = uv::UV_FS_O_NOCTTY as _;
        const NOFOLLOW = uv::UV_FS_O_NOFOLLOW as _;
        const NONBLOCK = uv::UV_FS_O_NONBLOCK as _;
        const RANDOM = uv::UV_FS_O_RANDOM as _;
        const RDONLY = uv::UV_FS_O_RDONLY as _;
        const RDWR = uv::UV_FS_O_RDWR as _;
        const SEQUENTIAL = uv::UV_FS_O_SEQUENTIAL as _;
        const SHORT_LIVED = uv::UV_FS_O_SHORT_LIVED as _;
        const SYMLINK = uv::UV_FS_O_SYMLINK as _;
        const SYNC = uv::UV_FS_O_SYNC as _;
        const TEMPORARY = uv::UV_FS_O_TEMPORARY as _;
        const TRUNC = uv::UV_FS_O_TRUNC as _;
        const WRONLY = uv::UV_FS_O_WRONLY as _;
    }
}

