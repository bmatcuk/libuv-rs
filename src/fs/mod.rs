//! All file operations are run on the threadpool. See Thread pool work scheduling for information
//! on the threadpool size.
//!
//! Note: Uses utf-8 encoding on Windows

include!("./fs_copy_flags.inc.rs");
include!("./fs_open_flags.inc.rs");
include!("./fs_mode_flags.rs");
include!("./fs_types.inc.rs");

use crate::{FromInner, FsReq, IntoInner};
use std::ffi::CString;
use uv::{
    uv_fs_access, uv_fs_chmod, uv_fs_chown, uv_fs_close, uv_fs_closedir, uv_fs_copyfile,
    uv_fs_fchmod, uv_fs_fchown, uv_fs_fdatasync, uv_fs_fstat, uv_fs_fsync, uv_fs_ftruncate,
    uv_fs_futime, uv_fs_get_ptr, uv_fs_get_result, uv_fs_get_statbuf, uv_fs_get_type, uv_fs_lchown,
    uv_fs_link, uv_fs_lstat, uv_fs_mkdir, uv_fs_mkdtemp, uv_fs_mkstemp, uv_fs_open, uv_fs_opendir,
    uv_fs_read, uv_fs_readdir, uv_fs_readlink, uv_fs_realpath, uv_fs_rename, uv_fs_rmdir,
    uv_fs_scandir, uv_fs_scandir_next, uv_fs_sendfile, uv_fs_stat, uv_fs_statfs, uv_fs_symlink,
    uv_fs_unlink, uv_fs_utime, uv_fs_write,
};

pub mod dir;
pub use dir::*;

pub mod dirent;
pub use dirent::*;

pub mod stat;
pub use stat::*;

pub mod statfs;
pub use statfs::*;

pub mod timespec;
pub use timespec::*;

type FsReqResult = crate::Result<FsReq>;
type FsReqErrResult = Result<FsReq, Box<dyn std::error::Error>>;
type SyncResult = crate::Result<isize>;
type SyncErrResult = Result<isize, Box<dyn std::error::Error>>;

/// Cross platform representation of a file handle.
pub type File = i32;

/// Destroys the given FsReq and returns the result
fn destroy_req_return_result(req: FsReq) -> isize {
    let result = req.result();
    req.destroy();
    result
}

impl crate::Loop {
    /// Private implementation for fs_close()
    fn _fs_close(&self, file: File, cb: Option<impl FnMut(FsReq) + 'static>) -> FsReqResult {
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_close(self.into_inner(), req.into_inner(), file as _, uv_cb)
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to close(2).
    pub fn fs_close(&self, file: File, cb: impl FnMut(FsReq) + 'static) -> FsReqResult {
        self._fs_close(file, Some(cb))
    }

    /// Equivalent to close(2).
    pub fn fs_close_sync(&self, file: File) -> SyncResult {
        self._fs_close(file, None::<fn(FsReq)> {})
            .map(destroy_req_return_result)
    }

    /// Private implementation for fs_open()
    fn _fs_open(
        &self,
        path: &str,
        flags: FsOpenFlags,
        mode: FsModeFlags,
        cb: Option<impl FnMut(FsReq) + 'static>,
    ) -> FsReqErrResult {
        let path = CString::new(path)?;
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_open(
                self.into_inner(),
                req.into_inner(),
                path.as_ptr(),
                flags.bits(),
                mode.bits(),
                uv_cb,
            )
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to open(2).
    ///
    /// Note: On Windows libuv uses CreateFileW and thus the file is always opened in binary mode.
    pub fn fs_open(
        &self,
        path: &str,
        flags: FsOpenFlags,
        mode: FsModeFlags,
        cb: impl FnMut(FsReq) + 'static,
    ) -> FsReqErrResult {
        self._fs_open(path, flags, mode, Some(cb))
    }

    /// Equivalent to open(2).
    ///
    /// Note: On Windows libuv uses CreateFileW and thus the file is always opened in binary mode.
    pub fn fs_open_sync(
        &self,
        path: &str,
        flags: FsOpenFlags,
        mode: FsModeFlags,
    ) -> Result<File, Box<dyn std::error::Error>> {
        self._fs_open(path, flags, mode, None::<fn(FsReq)>)
            .map(|req| {
                let file = req.file();
                req.destroy();
                return file as _;
            })
    }

    /// Private implementation for fs_read()
    fn _fs_read(
        &self,
        file: File,
        bufs: &[crate::Buf],
        offset: i64,
        cb: Option<impl FnMut(FsReq) + 'static>,
    ) -> FsReqResult {
        let req = FsReq::new(cb)?;
        let (bufs_ptr, bufs_len, bufs_capacity) = bufs.into_inner();
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_read(
                self.into_inner(),
                req.into_inner(),
                file as _,
                bufs_ptr as _,
                bufs_len as _,
                offset,
                uv_cb,
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to preadv(2).
    ///
    /// Warning: On Windows, under non-MSVC environments (e.g. when GCC or Clang is used to build
    /// libuv), files opened using the Filemap flag may cause a fatal crash if the memory mapped
    /// read operation fails.
    pub fn fs_read(
        &self,
        file: File,
        bufs: &[crate::Buf],
        offset: i64,
        cb: impl FnMut(FsReq) + 'static,
    ) -> FsReqResult {
        self._fs_read(file, bufs, offset, Some(cb))
    }

    /// Equivalent to preadv(2).
    ///
    /// Warning: On Windows, under non-MSVC environments (e.g. when GCC or Clang is used to build
    /// libuv), files opened using the Filemap flag may cause a fatal crash if the memory mapped
    /// read operation fails.
    pub fn fs_read_sync(&self, file: File, bufs: &[crate::Buf], offset: i64) -> SyncResult {
        self._fs_read(file, bufs, offset, None::<fn(FsReq)>)
            .map(destroy_req_return_result)
    }

    /// Private implementation for fs_unlink()
    fn _fs_unlink(&self, path: &str, cb: Option<impl FnMut(FsReq) + 'static>) -> FsReqErrResult {
        let path = CString::new(path)?;
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_unlink(self.into_inner(), req.into_inner(), path.as_ptr(), uv_cb)
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to unlink(2).
    pub fn fs_unlink(&self, path: &str, cb: impl FnMut(FsReq) + 'static) -> FsReqErrResult {
        self._fs_unlink(path, Some(cb))
    }

    /// Equivalent to unlink(2).
    pub fn fs_unlink_sync(&self, path: &str) -> SyncErrResult {
        self._fs_unlink(path, None::<fn(FsReq)>)
            .map(destroy_req_return_result)
    }

    /// Private implementation for fs_write()
    fn _fs_write(
        &self,
        file: File,
        bufs: &[impl crate::BufTrait],
        offset: i64,
        cb: Option<impl FnMut(FsReq) + 'static>,
    ) -> FsReqResult {
        let req = FsReq::new(cb)?;
        let (bufs_ptr, bufs_len, bufs_capacity) = bufs.into_inner();
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_write(
                self.into_inner(),
                req.into_inner(),
                file as _,
                bufs_ptr as _,
                bufs_len as _,
                offset,
                uv_cb,
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to pwritev(2).
    ///
    /// Warning: On Windows, under non-MSVC environments (e.g. when GCC or Clang is used to build
    /// libuv), files opened using the Filemap flag may cause a fatal crash if the memory mapped
    /// write operation fails.
    pub fn fs_write(
        &self,
        file: File,
        bufs: &[impl crate::BufTrait],
        offset: i64,
        cb: impl FnMut(FsReq) + 'static,
    ) -> FsReqResult {
        self._fs_write(file, bufs, offset, Some(cb))
    }

    /// Equivalent to pwritev(2).
    ///
    /// Warning: On Windows, under non-MSVC environments (e.g. when GCC or Clang is used to build
    /// libuv), files opened using the Filemap flag may cause a fatal crash if the memory mapped
    /// write operation fails.
    pub fn fs_write_sync(
        &self,
        file: File,
        bufs: &[impl crate::BufTrait],
        offset: i64,
    ) -> SyncResult {
        self._fs_write(file, bufs, offset, None::<fn(FsReq)>)
            .map(destroy_req_return_result)
    }

    /// Private implementation for fs_mkdir()
    fn _fs_mkdir(
        &self,
        path: &str,
        mode: FsModeFlags,
        cb: Option<impl FnMut(FsReq) + 'static>,
    ) -> FsReqErrResult {
        let path = CString::new(path)?;
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_mkdir(
                self.into_inner(),
                req.into_inner(),
                path.as_ptr() as _,
                mode.bits(),
                uv_cb,
            )
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to mkdir(2).
    ///
    /// Note: mode is currently not implemented on Windows.
    pub fn fs_mkdir(
        &self,
        path: &str,
        mode: FsModeFlags,
        cb: impl FnMut(FsReq) + 'static,
    ) -> FsReqErrResult {
        self._fs_mkdir(path, mode, Some(cb))
    }

    /// Equivalent to mkdir(2).
    ///
    /// Note: mode is currently not implemented on Windows.
    pub fn fs_mkdir_sync(&self, path: &str, mode: FsModeFlags) -> SyncErrResult {
        self._fs_mkdir(path, mode, None::<fn(FsReq)>)
            .map(destroy_req_return_result)
    }

    /// Private implementation for fs_mkdtemp()
    fn _fs_mkdtemp(&self, tpl: &str, cb: Option<impl FnMut(FsReq) + 'static>) -> FsReqErrResult {
        let tpl = CString::new(tpl)?;
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_mkdtemp(self.into_inner(), req.into_inner(), tpl.as_ptr(), uv_cb)
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy()
        }
        result.map(|_| req)
    }

    /// Equivalent to mkdtemp(3). The result can be found as req.path()
    pub fn fs_mkdtemp(&self, tpl: &str, cb: impl FnMut(FsReq) + 'static) -> FsReqErrResult {
        self._fs_mkdtemp(tpl, Some(cb))
    }

    /// Equivalent to mkdtemp(3).
    pub fn fs_mkdtemp_sync(&self, tpl: &str) -> Result<String, Box<dyn std::error::Error>> {
        self._fs_mkdtemp(tpl, None::<fn(FsReq)>).map(|req| {
            let path = req.path();
            req.destroy();
            return path;
        })
    }

    /// Private implementation for fs_rmdir()
    fn _fs_rmdir(&self, path: &str, cb: Option<impl FnMut(FsReq) + 'static>) -> FsReqErrResult {
        let path = CString::new(path)?;
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_rmdir(self.into_inner(), req.into_inner(), path.as_ptr(), uv_cb)
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to rmdir(2).
    pub fn fs_rmdir(&self, path: &str, cb: impl FnMut(FsReq) + 'static) -> FsReqErrResult {
        self._fs_rmdir(path, Some(cb))
    }

    /// Equivalent to rmdir(2).
    pub fn fs_rmdir_sync(&self, path: &str) -> SyncErrResult {
        self._fs_rmdir(path, None::<fn(FsReq)>)
            .map(destroy_req_return_result)
    }

    /// Private implementation for fs_opendir()
    fn _fs_opendir(&self, path: &str, cb: Option<impl FnMut(FsReq) + 'static>) -> FsReqErrResult {
        let path = CString::new(path)?;
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_opendir(self.into_inner(), req.into_inner(), path.as_ptr(), uv_cb)
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Opens path as a directory stream. On success, a Dir is allocated and returned via
    /// req.dir(). This memory is not freed by req.destroy(). The allocated memory must be freed by
    /// calling fs_closedir(). On failure, no memory is allocated.
    ///
    /// The contents of the directory can be iterated over by passing the resulting Dir to
    /// fs_readdir().
    pub fn fs_opendir(&self, path: &str, cb: impl FnMut(FsReq) + 'static) -> FsReqErrResult {
        self._fs_opendir(path, Some(cb))
    }

    /// Opens path as a directory stream. On success, a Dir is allocated and returned. The
    /// allocated memory must be freed by calling fs_closedir(). On failure, no memory is
    /// allocated.
    ///
    /// The contents of the directory can be iterated over by passing the resulting Dir to
    /// fs_readdir().
    pub fn fs_opendir_sync(&self, path: &str) -> Result<crate::Dir, Box<dyn std::error::Error>> {
        self._fs_opendir(path, None::<fn(FsReq)>).and_then(|req| {
            let dir = req.dir();
            req.destroy();
            dir.ok_or_else(|| Box::new(crate::Error::EINVAL) as _)
        })
    }

    /// Private implementation for fs_closedir()
    fn _fs_closedir(&self, dir: Dir, cb: Option<impl FnMut(FsReq) + 'static>) -> FsReqResult {
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_closedir(self.into_inner(), req.into_inner(), dir.into_inner(), uv_cb)
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Closes the directory stream represented by dir and frees the memory allocated by
    /// fs_opendir(). Don't forget to call Dir::free_entries() first!
    pub fn fs_closedir(&self, dir: Dir, cb: impl FnMut(FsReq) + 'static) -> FsReqResult {
        self._fs_closedir(dir, Some(cb))
    }

    /// Closes the directory stream represented by dir and frees the memory allocated by
    /// fs_opendir(). Don't forget to call Dir::free_entries() first!
    pub fn fs_closedir_sync(&self, dir: Dir) -> SyncResult {
        self._fs_closedir(dir, None::<fn(FsReq)>)
            .map(destroy_req_return_result)
    }

    /// Private implementation for fs_readdir
    fn _fs_readdir(&self, dir: &Dir, cb: Option<impl FnMut(FsReq) + 'static>) -> FsReqResult {
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_readdir(
                self.into_inner(),
                req.into_inner(),
                (*dir).into_inner(),
                uv_cb,
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Iterates over the directory stream, dir, returned by a successful fs_opendir() call. Prior
    /// to invoking fs_readdir(), the caller must allocate space for directory entries by calling
    /// Dir::reserve().
    ///
    /// Warning: fs_readdir() is not thread safe.
    ///
    /// Note: This function does not return the “.” and “..” entries.
    ///
    /// Note: On success this function allocates memory that must be freed using FsReq::destroy().
    /// destroy() must be called before closing the directory with fs_closedir().
    pub fn fs_readdir(&self, dir: &Dir, cb: impl FnMut(FsReq) + 'static) -> FsReqResult {
        self._fs_readdir(dir, Some(cb)).and_then(|req| {
            if let Some(dir) = req.dir() {
                dir.set_len(req.result() as _);
            }
            Ok(req)
        })
    }

    /// Iterates over the directory stream, dir, returned by a successful fs_opendir() call. Prior
    /// to invoking fs_readdir(), the caller must allocate space for directory entries by calling
    /// Dir::reserve().
    ///
    /// On success, the result is an integer >= 0 representing the number of entries read from the
    /// stream.
    ///
    /// Warning: fs_readdir() is not thread safe.
    ///
    /// Note: This function does not return the “.” and “..” entries.
    pub fn fs_readdir_sync(&self, dir: &Dir) -> SyncResult {
        self._fs_readdir(dir, None::<fn(FsReq)>)
            .map(destroy_req_return_result)
    }

    /// Private implementation for fs_scandir()
    fn _fs_scandir(
        &self,
        path: &str,
        flags: FsOpenFlags,
        cb: Option<impl FnMut(FsReq) + 'static>,
    ) -> FsReqErrResult {
        let path = CString::new(path)?;
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_scandir(
                self.into_inner(),
                req.into_inner(),
                path.as_ptr(),
                flags.bits(),
                uv_cb,
            )
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Start scanning a directory. Unlike most other fs_* methods, the callback is passed a
    /// ScandirIter which is an iterator over the entries in the directory.
    ///
    /// Note: Unlike scandir(3), this function does not return the “.” and “..” entries.
    ///
    /// Note: On Linux, getting the type of an entry is only supported by some file systems (btrfs,
    /// ext2, ext3 and ext4 at the time of this writing), check the getdents(2) man page.
    pub fn fs_scandir(
        &self,
        path: &str,
        flags: FsOpenFlags,
        cb: impl FnMut(&FsReq, ScandirIter) + 'static,
    ) -> FsReqErrResult {
        self._fs_scandir(path, flags, Some(|req| cb(&req, ScandirIter { req })))
    }

    /// Returns a ScandirIter that can be used to iterate over the contents of a directory.
    pub fn fs_scandir_sync(
        &self,
        path: &str,
        flags: FsOpenFlags,
    ) -> Result<ScandirIter, Box<dyn std::error::Error>> {
        self._fs_scandir(path, flags, None::<fn(FsReq)>)
            .map(|req| ScandirIter { req })
    }

    /// Private implementation for fs_stat()
    fn _fs_stat(&self, path: &str, cb: Option<impl FnMut(FsReq) + 'static>) -> FsReqErrResult {
        let path = CString::new(path)?;
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_stat(self.into_inner(), req.into_inner(), path.as_ptr(), uv_cb)
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to stat(2).
    pub fn fs_stat(&self, path: &str, cb: impl FnMut(FsReq) + 'static) -> FsReqErrResult {
        self._fs_stat(path, Some(cb))
    }

    /// Equivalent to stat(2).
    pub fn fs_stat_sync(&self, path: &str) -> Result<Stat, Box<dyn std::error::Error>> {
        self._fs_stat(path, None::<fn(FsReq)>).map(|req| {
            let stat = req.stat();
            req.destroy();
            return stat;
        })
    }

    /// Private implementation for fs_fstat()
    fn _fs_fstat(&self, file: File, cb: Option<impl FnMut(FsReq) + 'static>) -> FsReqResult {
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_fstat(self.into_inner(), req.into_inner(), file as _, uv_cb)
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to fstat(2).
    pub fn fs_fstat(&self, file: File, cb: impl FnMut(FsReq) + 'static) -> FsReqResult {
        self._fs_fstat(file, Some(cb))
    }

    /// Equivalent to fstat(2).
    pub fn fs_fstat_sync(&self, file: File) -> crate::Result<Stat> {
        self._fs_fstat(file, None::<fn(FsReq)>).map(|req| {
            let stat = req.stat();
            req.destroy();
            return stat;
        })
    }

    /// Private implementation for fs_lstat
    fn _fs_lstat(&self, path: &str, cb: Option<impl FnMut(FsReq) + 'static>) -> FsReqErrResult {
        let path = CString::new(path)?;
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_lstat(self.into_inner(), req.into_inner(), path.as_ptr(), uv_cb)
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to lstat(2).
    pub fn fs_lstat(&self, path: &str, cb: impl FnMut(FsReq) + 'static) -> FsReqErrResult {
        self._fs_lstat(path, Some(cb))
    }

    /// Equivalent to lstat(2).
    pub fn fs_lstat_sync(&self, path: &str) -> Result<Stat, Box<dyn std::error::Error>> {
        self._fs_lstat(path, None::<fn(FsReq)>).map(|req| {
            let stat = req.stat();
            req.destroy();
            return stat;
        })
    }

    /// Private implementation for fs_statfs()
    fn _fs_statfs(&self, path: &str, cb: Option<impl FnMut(FsReq) + 'static>) -> FsReqErrResult {
        let path = CString::new(path)?;
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_statfs(self.into_inner(), req.into_inner(), path.as_ptr(), uv_cb)
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to statfs(2). On success, FsReq::statfs() will return a StatFs
    ///
    /// Note: Any fields in the resulting StatFs that are not supported by the underlying operating
    /// system are set to zero.
    pub fn fs_statfs(&self, path: &str, cb: impl FnMut(FsReq) + 'static) -> FsReqErrResult {
        self._fs_statfs(path, Some(cb))
    }

    /// Equivalent to statfs(2). On success, FsReq::statfs() will return a StatFs
    ///
    /// Note: Any fields in the resulting StatFs that are not supported by the underlying operating
    /// system are set to zero.
    pub fn fs_statfs_sync(&self, path: &str) -> Result<StatFs, Box<dyn std::error::Error>> {
        self._fs_statfs(path, None::<fn(FsReq)>).and_then(|req| {
            let statfs = req.statfs();
            req.destroy();
            statfs.ok_or_else(|| Box::new(crate::Error::EINVAL) as _)
        })
    }

    /// Private implementation for fs_rename()
    fn _fs_rename(
        &self,
        path: &str,
        new_path: &str,
        cb: Option<impl FnMut(FsReq) + 'static>,
    ) -> FsReqErrResult {
        let path = CString::new(path)?;
        let new_path = CString::new(new_path)?;
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_rename(
                self.into_inner(),
                req.into_inner(),
                path.as_ptr(),
                new_path.as_ptr(),
                uv_cb,
            )
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to rename(2).
    pub fn fs_rename(
        &self,
        path: &str,
        new_path: &str,
        cb: impl FnMut(FsReq) + 'static,
    ) -> FsReqErrResult {
        self._fs_rename(path, new_path, Some(cb))
    }

    /// Equivalent to rename(2).
    pub fn fs_rename_sync(&self, path: &str, new_path: &str) -> SyncErrResult {
        self._fs_rename(path, new_path, None::<fn(FsReq)>)
            .map(destroy_req_return_result)
    }

    /// Private implementation for fs_fsync()
    fn _fs_fsync(&self, file: File, cb: Option<impl FnMut(FsReq) + 'static>) -> FsReqResult {
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_fsync(self.into_inner(), req.into_inner(), file as _, uv_cb)
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to fsync(2).
    ///
    /// Note: For AIX, uv_fs_fsync returns UV_EBADF on file descriptors referencing non regular
    /// files.
    pub fn fs_fsync(&self, file: File, cb: impl FnMut(FsReq) + 'static) -> FsReqResult {
        self._fs_fsync(file, Some(cb))
    }

    /// Equivalent to fsync(2).
    ///
    /// Note: For AIX, uv_fs_fsync returns UV_EBADF on file descriptors referencing non regular
    /// files.
    pub fn fs_fsync_sync(&self, file: File) -> SyncResult {
        self._fs_fsync(file, None::<fn(FsReq)>)
            .map(destroy_req_return_result)
    }

    /// Private implementation for fs_fdatasync()
    fn _fs_fdatasync(&self, file: File, cb: Option<impl FnMut(FsReq) + 'static>) -> FsReqResult {
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_fdatasync(self.into_inner(), req.into_inner(), file as _, uv_cb)
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to fdatasync(2).
    pub fn fs_fdatasync(&self, file: File, cb: impl FnMut(FsReq) + 'static) -> FsReqResult {
        self._fs_fdatasync(file, Some(cb))
    }

    /// Equivalent to fdatasync(2).
    pub fn fs_fdatasync_sync(&self, file: File) -> SyncResult {
        self._fs_fdatasync(file, None::<fn(FsReq)>)
            .map(destroy_req_return_result)
    }

    /// Private implementation for fs_ftruncate()
    fn _fs_ftruncate(
        &self,
        file: File,
        offset: i64,
        cb: Option<impl FnMut(FsReq) + 'static>,
    ) -> FsReqResult {
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_ftruncate(
                self.into_inner(),
                req.into_inner(),
                file as _,
                offset,
                uv_cb,
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to ftruncate(2).
    pub fn fs_ftruncate(
        &self,
        file: File,
        offset: i64,
        cb: impl FnMut(FsReq) + 'static,
    ) -> FsReqResult {
        self._fs_ftruncate(file, offset, Some(cb))
    }

    /// Equivalent to ftruncate(2).
    pub fn fs_ftruncate_sync(&self, file: File, offset: i64) -> SyncResult {
        self._fs_ftruncate(file, offset, None::<fn(FsReq)>)
            .map(destroy_req_return_result)
    }

    /// Private implementation for fs_copyfile()
    fn _fs_copyfile(
        &self,
        path: &str,
        new_path: &str,
        flags: FsCopyFlags,
        cb: Option<impl FnMut(FsReq) + 'static>,
    ) -> FsReqErrResult {
        let path = CString::new(path)?;
        let new_path = CString::new(path)?;
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_copyfile(
                self.into_inner(),
                req.into_inner(),
                path.as_ptr(),
                new_path.as_ptr(),
                flags.bits(),
                uv_cb,
            )
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Copies a file from path to new_path. Supported flags are described below.
    ///
    ///   * EXCL: If present, fs_copyfile() will fail with EEXIST if the destination path already
    ///     exists. The default behavior is to overwrite the destination if it exists.
    ///   * FICLONE: If present, fs_copyfile() will attempt to create a copy-on-write reflink. If
    ///     the underlying platform does not support copy-on-write, or an error occurs while
    ///     attempting to use copy-on-write, a fallback copy mechanism based on fs_sendfile() is
    ///     used.
    ///   * FICLONE_FORCE: If present, fs_copyfile() will attempt to create a copy-on-write
    ///     reflink. If the underlying platform does not support copy-on-write, or an error occurs
    ///     while attempting to use copy-on-write, then an error is returned.
    ///
    /// Warning: If the destination path is created, but an error occurs while copying the data,
    /// then the destination path is removed. There is a brief window of time between closing and
    /// removing the file where another process could access the file.
    pub fn fs_copyfile(
        &self,
        path: &str,
        new_path: &str,
        flags: FsCopyFlags,
        cb: impl FnMut(FsReq) + 'static,
    ) -> FsReqErrResult {
        self._fs_copyfile(path, new_path, flags, Some(cb))
    }

    /// Copies a file from path to new_path. Supported flags are described below.
    ///
    ///   * EXCL: If present, fs_copyfile() will fail with EEXIST if the destination path already
    ///     exists. The default behavior is to overwrite the destination if it exists.
    ///   * FICLONE: If present, fs_copyfile() will attempt to create a copy-on-write reflink. If
    ///     the underlying platform does not support copy-on-write, or an error occurs while
    ///     attempting to use copy-on-write, a fallback copy mechanism based on fs_sendfile() is
    ///     used.
    ///   * FICLONE_FORCE: If present, fs_copyfile() will attempt to create a copy-on-write
    ///     reflink. If the underlying platform does not support copy-on-write, or an error occurs
    ///     while attempting to use copy-on-write, then an error is returned.
    ///
    /// Warning: If the destination path is created, but an error occurs while copying the data,
    /// then the destination path is removed. There is a brief window of time between closing and
    /// removing the file where another process could access the file.
    pub fn fs_copyfile_sync(
        &self,
        path: &str,
        new_path: &str,
        flags: FsCopyFlags,
    ) -> SyncErrResult {
        self._fs_copyfile(path, new_path, flags, None::<fn(FsReq)>)
            .map(destroy_req_return_result)
    }

    /// Private implementation for fs_sendfile
    fn _fs_sendfile(
        &self,
        out_file: File,
        in_file: File,
        offset: i64,
        len: usize,
        cb: Option<impl FnMut(FsReq) + 'static>,
    ) -> FsReqResult {
        let req = FsReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| crate::uv_fs_cb as _);
        let result = crate::uvret(unsafe {
            uv_fs_sendfile(
                self.into_inner(),
                req.into_inner(),
                out_file as _,
                in_file as _,
                offset,
                len,
                uv_cb,
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Limited equivalent to sendfile(2).
    pub fn fs_sendfile(
        &self,
        out_file: File,
        in_file: File,
        offset: i64,
        len: usize,
        cb: impl FnMut(FsReq) + 'static,
    ) -> FsReqResult {
        self._fs_sendfile(out_file, in_file, offset, len, Some(cb))
    }

    /// Limited equivalent to sendfile(2).
    pub fn fs_sendfile_sync(
        &self,
        out_file: File,
        in_file: File,
        offset: i64,
        len: usize,
    ) -> SyncResult {
        self._fs_sendfile(out_file, in_file, offset, len, None::<fn(FsReq)>)
            .map(destroy_req_return_result)
    }
}

/// An iterator using scandir to get a directory listing.
///
/// Note: Unlike scandir(3), this function does not return the “.” and “..” entries.
///
/// Note: On Linux, getting the type of an entry is only supported by some file systems (btrfs,
/// ext2, ext3 and ext4 at the time of this writing), check the getdents(2) man page.
pub struct ScandirIter {
    req: FsReq,
}

impl Iterator for ScandirIter {
    type Item = crate::Result<crate::Dirent>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut dirent: uv::uv_dirent_t = std::mem::zeroed();
        let result =
            crate::uvret(unsafe { uv_fs_scandir_next(self.req.into_inner(), &mut dirent as _) });
        match result {
            Ok(_) => Some(Ok(crate::Dirent::from_inner(
                &dirent as *const uv::uv_dirent_t,
            ))),
            Err(crate::Error::EOF) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

impl std::iter::FusedIterator for ScandirIter {}

impl Drop for ScandirIter {
    fn drop(&mut self) {
        self.req.destroy();
    }
}
