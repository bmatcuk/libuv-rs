use crate::FromInner;
use uv::uv_statfs_t;

/// Reduced cross platform equivalent of struct statfs. Used in statfs()
pub struct StatFs {
    pub r#type: u64,
    pub bsize: u64,
    pub blocks: u64,
    pub bfree: u64,
    pub bavail: u64,
    pub files: u64,
    pub ffree: u64,
    pub frsize: u64,
    pub spare: [u64; 3usize],
}

impl FromInner<*mut uv_statfs_t> for StatFs {
    fn from_inner(statfs: *mut uv_statfs_t) -> StatFs {
        unsafe {
            StatFs {
                r#type: (*statfs).f_type,
                bsize: (*statfs).f_bsize,
                blocks: (*statfs).f_blocks,
                bfree: (*statfs).f_bfree,
                bavail: (*statfs).f_bavail,
                files: (*statfs).f_files,
                ffree: (*statfs).f_ffree,
                frsize: (*statfs).f_frsize,
                spare: (*statfs).f_spare,
            }
        }
    }
}
