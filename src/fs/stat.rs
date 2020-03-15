use crate::{FromInner, IntoInner};
use uv::uv_stat_t;

/// Portable equivalent of struct stat.
pub struct Stat {
    pub dev: u64,
    pub mode: u64,
    pub nlink: u64,
    pub uid: u64,
    pub gid: u64,
    pub rdev: u64,
    pub ino: u64,
    pub size: u64,
    pub blksize: u64,
    pub blocks: u64,
    pub flags: u64,
    pub gen: u64,
    pub atim: crate::TimeSpec,
    pub mtim: crate::TimeSpec,
    pub ctim: crate::TimeSpec,
    pub birthtim: crate::TimeSpec,
}

impl FromInner<uv_stat_t> for Stat {
    fn from_inner(stat: uv_stat_t) -> Stat {
        Stat {
            dev: stat.st_dev,
            mode: stat.st_mode,
            nlink: stat.st_nlink,
            uid: stat.st_uid,
            gid: stat.st_gid,
            rdev: stat.st_rdev,
            ino: stat.st_ino,
            size: stat.st_size,
            blksize: stat.st_blksize,
            blocks: stat.st_blocks,
            flags: stat.st_flags,
            gen: stat.st_gen,
            atim: stat.st_atim.into_inner(),
            mtim: stat.st_mtim.into_inner(),
            ctim: stat.st_ctim.into_inner(),
            birthtim: stat.st_birthtim.into_inner(),
        }
    }
}
