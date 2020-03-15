#[allow(non_camel_case_types)]
bitflags! {
    pub struct FsCopyFlags: i32 {
        const EXCL = uv::UV_FS_COPYFILE_EXCL as _;
        const FICLONE = uv::UV_FS_COPYFILE_FICLONE as _;
        const FICLONE_FORCE = uv::UV_FS_COPYFILE_FICLONE_FORCE as _;
    }
}

