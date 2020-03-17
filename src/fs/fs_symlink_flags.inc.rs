#[allow(non_camel_case_types)]
bitflags! {
    pub struct FsSymlinkFlags: i32 {
        const DIR = uv::UV_FS_SYMLINK_DIR as _;
        const JUNCTION = uv::UV_FS_SYMLINK_JUNCTION as _;
    }
}

