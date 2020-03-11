include!("./fs_types.inc.rs");

use crate::{FromInner, IntoInner};
use uv::{
    uv_fs_access, uv_fs_chmod, uv_fs_chown, uv_fs_close, uv_fs_closedir, uv_fs_copyfile,
    uv_fs_fchmod, uv_fs_fchown, uv_fs_fdatasync, uv_fs_fstat, uv_fs_fsync, uv_fs_ftruncate,
    uv_fs_futime, uv_fs_get_ptr, uv_fs_get_result, uv_fs_get_statbuf, uv_fs_get_type, uv_fs_lchown,
    uv_fs_link, uv_fs_lstat, uv_fs_mkdir, uv_fs_mkdtemp, uv_fs_mkstemp, uv_fs_open, uv_fs_opendir,
    uv_fs_read, uv_fs_readdir, uv_fs_readlink, uv_fs_realpath, uv_fs_rename, uv_fs_rmdir,
    uv_fs_scandir, uv_fs_scandir_next, uv_fs_sendfile, uv_fs_stat, uv_fs_statfs, uv_fs_symlink,
    uv_fs_unlink, uv_fs_utime, uv_fs_write,
};

pub mod dirent;
pub use dirent::*;

pub mod stat;
pub use stat::*;

pub mod statfs;
pub use statfs::*;

pub mod timespec;
pub use timespec::*;

/// Cross platform representation of a file handle.
pub type File = i32;

bitflags! {
    pub struct FsOpenFlags {
        const Append = O_APPEND;
    }
}

impl crate::Loop {
    /// Equivalent to close(2).
    ///
    /// If the callback is None the request is completed synchronously, otherwise it will be
    /// performed asynchronously. If performed synchronously, the returned FsReq can be ignored.
    ///
    /// All file operations are run on the threadpool. See Thread pool work scheduling for
    /// information on the threadpool size.
    ///
    /// Note: Uses utf-8 encoding on Windows
    pub fn fs_close(
        &self,
        file: File,
        cb: Option<impl FnMut(crate::FsReq) + 'static>,
    ) -> crate::Result<crate::FsReq> {
        let req = crate::FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_close(self.into_inner(), req.into_inner(), file as _, uv_cb)
        });
        if result.is_err() || uv_cb.is_none() {
            req.destroy();
        }
        result.map(|_| req)
    }

    pub fn fs_open(
        &self,
        path: &str,
        flags: FsOpenFlags,
        mode: FsModeFlags,
        cb: Option<impl FnMut(crate::FsReq) + 'static>,
    ) -> crate::Result<crate::FsReq> {
    }
}
