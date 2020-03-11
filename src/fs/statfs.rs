use crate::{FromInner, IntoInner};
use uv::uv_statfs_t;

pub struct StatFs {
    pub r#type: u64,
    pub bsize: u64,
    pub blocks: u64,
    pub bfree: u64,
    pub bavail: u64,
    pub files: u64,
    pub ffree: u64,
    pub spare: [u64; 4usize],
}

impl FromInner<*const uv_statfs_t> for StatFs {
    fn from_inner(statfs: *const uv_statfs_t) -> StatFs {
        StatFs {
            r#type: (*statfs).f_type,
            bsize: (*statfs).f_bsize,
            blocks: (*statfs).f_blocks,
            bfree: (*statfs).f_bfree,
            bavail: (*statfs).f_bavail,
            files: (*statfs).f_files,
            ffree: (*statfs).f_ffree,
            spare: (*statfs).f_spare,
        }
    }
}
