//! All file operations are run on the threadpool. See Thread pool work scheduling for information
//! on the threadpool size.
//!
//! Note: Uses utf-8 encoding on Windows

include!("./fs_copy_flags.inc.rs");
include!("./fs_open_flags.inc.rs");
include!("./fs_mode_flags.rs");
include!("./fs_symlink_flags.inc.rs");
include!("./fs_types.inc.rs");

use crate::{FromInner, FsReq, Inner, IntoInner};
use std::ffi::CString;
use uv::{
    uv_fs_access, uv_fs_chmod, uv_fs_chown, uv_fs_close, uv_fs_closedir, uv_fs_copyfile,
    uv_fs_fchmod, uv_fs_fchown, uv_fs_fdatasync, uv_fs_fstat, uv_fs_fsync, uv_fs_ftruncate,
    uv_fs_futime, uv_fs_lchown, uv_fs_link, uv_fs_lstat, uv_fs_mkdir, uv_fs_mkdtemp, uv_fs_mkstemp,
    uv_fs_open, uv_fs_opendir, uv_fs_read, uv_fs_readdir, uv_fs_readlink, uv_fs_realpath,
    uv_fs_rename, uv_fs_rmdir, uv_fs_scandir, uv_fs_scandir_next, uv_fs_sendfile, uv_fs_stat,
    uv_fs_statfs, uv_fs_symlink, uv_fs_unlink, uv_fs_utime, uv_fs_write,
};

pub mod dir;
pub use dir::*;

pub mod dirent;
pub use dirent::*;

pub mod misc;
pub use misc::*;

pub mod stat;
pub use stat::*;

pub mod statfs;
pub use statfs::*;

pub mod timespec;
pub use timespec::*;

type FsReqResult = crate::Result<FsReq>;
type FsReqErrResult = Result<FsReq, Box<dyn std::error::Error>>;
type SyncResult = crate::Result<usize>;
type SyncErrResult = Result<usize, Box<dyn std::error::Error>>;

/// Cross platform representation of a file handle.
pub type File = i32;

/// Platform dependent representation of a file handle.
#[cfg(windows)]
pub type OsFile = *mut std::ffi::c_void;
#[cfg(not(windows))]
pub type OsFile = i32;

/// Cross platform representation of a socket handle.
#[cfg(windows)]
pub type Socket = u64;
#[cfg(not(windows))]
pub type Socket = i32;

/// Cross platform representation of a user id
#[cfg(windows)]
pub type Uid = u8;
#[cfg(not(windows))]
pub type Uid = u32;

/// Cross platform representation of a group id
#[cfg(windows)]
pub type Gid = u8;
#[cfg(not(windows))]
pub type Gid = u32;

/// Destroys the given FsReq and returns the result
fn destroy_req_return_result(mut req: FsReq) -> SyncResult {
    let result = req.result();
    req.destroy();
    result
}

/// Destroys the given FsReq and returns the result
fn destroy_req_return_boxed_result(req: FsReq) -> SyncErrResult {
    destroy_req_return_result(req).map_err(|e| Box::new(e) as _)
}

impl crate::Loop {
    /// Private implementation for fs_close()
    fn _fs_close<CB: Into<crate::FsCB<'static>>>(&self, file: File, cb: CB) -> FsReqResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let mut req = FsReq::new(cb)?;
        let result =
            crate::uvret(unsafe { uv_fs_close(self.into_inner(), req.inner(), file as _, uv_cb) });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to close(2).
    pub fn fs_close<CB: Into<crate::FsCB<'static>>>(&self, file: File, cb: CB) -> FsReqResult {
        self._fs_close(file, cb)
    }

    /// Equivalent to close(2).
    pub fn fs_close_sync(&self, file: File) -> SyncResult {
        self._fs_close(file, ()).and_then(destroy_req_return_result)
    }

    /// Private implementation for fs_open()
    fn _fs_open<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        flags: FsOpenFlags,
        mode: FsModeFlags,
        cb: CB,
    ) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_open(
                self.into_inner(),
                req.inner(),
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
    pub fn fs_open<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        flags: FsOpenFlags,
        mode: FsModeFlags,
        cb: CB,
    ) -> FsReqErrResult {
        self._fs_open(path, flags, mode, cb)
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
        self._fs_open(path, flags, mode, ()).and_then(|mut req| {
            let file = req.result();
            req.destroy();
            file.map(|f| f as _).map_err(|e| Box::new(e) as _)
        })
    }

    /// Private implementation for fs_read()
    fn _fs_read<CB: Into<crate::FsCB<'static>>>(
        &self,
        file: File,
        bufs: &[crate::Buf],
        offset: i64,
        cb: CB,
    ) -> FsReqResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let mut req = FsReq::new(cb)?;
        let (bufs_ptr, bufs_len, _) = bufs.into_inner();
        let result = crate::uvret(unsafe {
            uv_fs_read(
                self.into_inner(),
                req.inner(),
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
    pub fn fs_read<CB: Into<crate::FsCB<'static>>>(
        &self,
        file: File,
        bufs: &[crate::Buf],
        offset: i64,
        cb: CB,
    ) -> FsReqResult {
        self._fs_read(file, bufs, offset, cb)
    }

    /// Equivalent to preadv(2).
    ///
    /// Warning: On Windows, under non-MSVC environments (e.g. when GCC or Clang is used to build
    /// libuv), files opened using the Filemap flag may cause a fatal crash if the memory mapped
    /// read operation fails.
    pub fn fs_read_sync(&self, file: File, bufs: &[crate::Buf], offset: i64) -> SyncResult {
        self._fs_read(file, bufs, offset, ())
            .and_then(destroy_req_return_result)
    }

    /// Private implementation for fs_unlink()
    fn _fs_unlink<CB: Into<crate::FsCB<'static>>>(&self, path: &str, cb: CB) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_unlink(self.into_inner(), req.inner(), path.as_ptr(), uv_cb)
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to unlink(2).
    pub fn fs_unlink<CB: Into<crate::FsCB<'static>>>(&self, path: &str, cb: CB) -> FsReqErrResult {
        self._fs_unlink(path, cb)
    }

    /// Equivalent to unlink(2).
    pub fn fs_unlink_sync(&self, path: &str) -> SyncErrResult {
        self._fs_unlink(path, ())
            .and_then(destroy_req_return_boxed_result)
    }

    /// Private implementation for fs_write()
    fn _fs_write<CB: Into<crate::FsCB<'static>>>(
        &self,
        file: File,
        bufs: &[impl crate::BufTrait],
        offset: i64,
        cb: CB,
    ) -> FsReqResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let mut req = FsReq::new(cb)?;
        let (bufs_ptr, bufs_len, _) = bufs.into_inner();
        let result = crate::uvret(unsafe {
            uv_fs_write(
                self.into_inner(),
                req.inner(),
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
    pub fn fs_write<CB: Into<crate::FsCB<'static>>>(
        &self,
        file: File,
        bufs: &[impl crate::BufTrait],
        offset: i64,
        cb: CB,
    ) -> FsReqResult {
        self._fs_write(file, bufs, offset, cb)
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
        self._fs_write(file, bufs, offset, ())
            .and_then(destroy_req_return_result)
    }

    /// Private implementation for fs_mkdir()
    fn _fs_mkdir<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        mode: FsModeFlags,
        cb: CB,
    ) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_mkdir(
                self.into_inner(),
                req.inner(),
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
    pub fn fs_mkdir<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        mode: FsModeFlags,
        cb: CB,
    ) -> FsReqErrResult {
        self._fs_mkdir(path, mode, cb)
    }

    /// Equivalent to mkdir(2).
    ///
    /// Note: mode is currently not implemented on Windows.
    pub fn fs_mkdir_sync(&self, path: &str, mode: FsModeFlags) -> SyncErrResult {
        self._fs_mkdir(path, mode, ())
            .and_then(destroy_req_return_boxed_result)
    }

    /// Private implementation for fs_mkdtemp()
    fn _fs_mkdtemp<CB: Into<crate::FsCB<'static>>>(&self, tpl: &str, cb: CB) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let tpl = CString::new(tpl)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_mkdtemp(self.into_inner(), req.inner(), tpl.as_ptr(), uv_cb)
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy()
        }
        result.map(|_| req)
    }

    /// Equivalent to mkdtemp(3). The result can be found as req.path()
    pub fn fs_mkdtemp<CB: Into<crate::FsCB<'static>>>(&self, tpl: &str, cb: CB) -> FsReqErrResult {
        self._fs_mkdtemp(tpl, cb)
    }

    /// Equivalent to mkdtemp(3).
    pub fn fs_mkdtemp_sync(&self, tpl: &str) -> Result<String, Box<dyn std::error::Error>> {
        self._fs_mkdtemp(tpl, ()).map(|mut req| {
            let path = req.path();
            req.destroy();
            return path;
        })
    }

    /// Private implementation for fs_mkstemp()
    fn _fs_mkstemp<CB: Into<crate::FsCB<'static>>>(&self, tpl: &str, cb: CB) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let tpl = CString::new(tpl)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_mkstemp(self.into_inner(), req.inner(), tpl.as_ptr(), uv_cb)
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to mkstemp(3).
    pub fn fs_mkstemp<CB: Into<crate::FsCB<'static>>>(&self, tpl: &str, cb: CB) -> FsReqErrResult {
        self._fs_mkstemp(tpl, cb)
    }

    /// Equivalent to mkstemp(3).
    pub fn fs_mkstemp_sync(&self, tpl: &str) -> SyncErrResult {
        self._fs_mkstemp(tpl, ())
            .and_then(destroy_req_return_boxed_result)
    }

    /// Private implementation for fs_rmdir()
    fn _fs_rmdir<CB: Into<crate::FsCB<'static>>>(&self, path: &str, cb: CB) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_rmdir(self.into_inner(), req.inner(), path.as_ptr(), uv_cb)
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to rmdir(2).
    pub fn fs_rmdir<CB: Into<crate::FsCB<'static>>>(&self, path: &str, cb: CB) -> FsReqErrResult {
        self._fs_rmdir(path, cb)
    }

    /// Equivalent to rmdir(2).
    pub fn fs_rmdir_sync(&self, path: &str) -> SyncErrResult {
        self._fs_rmdir(path, ())
            .and_then(destroy_req_return_boxed_result)
    }

    /// Private implementation for fs_opendir()
    fn _fs_opendir<CB: Into<crate::FsCB<'static>>>(&self, path: &str, cb: CB) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_opendir(self.into_inner(), req.inner(), path.as_ptr(), uv_cb)
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
    pub fn fs_opendir<CB: Into<crate::FsCB<'static>>>(&self, path: &str, cb: CB) -> FsReqErrResult {
        self._fs_opendir(path, cb)
    }

    /// Opens path as a directory stream. On success, a Dir is allocated and returned. The
    /// allocated memory must be freed by calling fs_closedir(). On failure, no memory is
    /// allocated.
    ///
    /// The contents of the directory can be iterated over by passing the resulting Dir to
    /// fs_readdir().
    pub fn fs_opendir_sync(&self, path: &str) -> Result<crate::Dir, Box<dyn std::error::Error>> {
        self._fs_opendir(path, ()).and_then(|mut req| {
            let dir = req.dir();
            req.destroy();
            dir.ok_or_else(|| Box::new(crate::Error::EINVAL) as _)
        })
    }

    /// Private implementation for fs_closedir()
    fn _fs_closedir<CB: Into<crate::FsCB<'static>>>(&self, dir: &Dir, cb: CB) -> FsReqResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_closedir(self.into_inner(), req.inner(), dir.into_inner(), uv_cb)
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Closes the directory stream represented by dir and frees the memory allocated by
    /// fs_opendir(). Don't forget to call Dir::free_entries() first!
    pub fn fs_closedir<CB: Into<crate::FsCB<'static>>>(&self, dir: &Dir, cb: CB) -> FsReqResult {
        self._fs_closedir(dir, cb)
    }

    /// Closes the directory stream represented by dir and frees the memory allocated by
    /// fs_opendir(). Don't forget to call Dir::free_entries() first!
    pub fn fs_closedir_sync(&self, dir: &Dir) -> SyncResult {
        self._fs_closedir(dir, ())
            .and_then(destroy_req_return_result)
    }

    /// Private implementation for fs_readdir
    fn _fs_readdir<CB: Into<crate::FsCB<'static>>>(&self, dir: &Dir, cb: CB) -> FsReqResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_readdir(self.into_inner(), req.inner(), dir.into_inner(), uv_cb)
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
    pub fn fs_readdir<CB: Into<crate::FsCB<'static>>>(&self, dir: &Dir, cb: CB) -> FsReqResult {
        self._fs_readdir(dir, cb).and_then(|req| {
            if let Some(dir) = req.dir().as_mut() {
                let result = req.result()?;
                dir.set_len(result as _);
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
        self._fs_readdir(dir, ())
            .and_then(destroy_req_return_result)
    }

    /// Private implementation for fs_scandir()
    fn _fs_scandir<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        flags: FsOpenFlags,
        cb: CB,
    ) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_scandir(
                self.into_inner(),
                req.inner(),
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
    /// ScandirIter which is an iterator over the entries in the directory. If you need access to
    /// the FsReq in the callback, you can access iter.req.
    ///
    /// Note: Unlike scandir(3), this function does not return the “.” and “..” entries.
    ///
    /// Note: On Linux, getting the type of an entry is only supported by some file systems (btrfs,
    /// ext2, ext3 and ext4 at the time of this writing), check the getdents(2) man page.
    pub fn fs_scandir(
        &self,
        path: &str,
        flags: FsOpenFlags,
        mut cb: impl FnMut(ScandirIter) + 'static,
    ) -> FsReqErrResult {
        self._fs_scandir(path, flags, move |req| cb(ScandirIter { req }))
    }

    /// Returns a ScandirIter that can be used to iterate over the contents of a directory.
    pub fn fs_scandir_sync(
        &self,
        path: &str,
        flags: FsOpenFlags,
    ) -> Result<ScandirIter, Box<dyn std::error::Error>> {
        self._fs_scandir(path, flags, ())
            .map(|req| ScandirIter { req })
    }

    /// Private implementation for fs_stat()
    fn _fs_stat<CB: Into<crate::FsCB<'static>>>(&self, path: &str, cb: CB) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_stat(self.into_inner(), req.inner(), path.as_ptr(), uv_cb)
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to stat(2).
    pub fn fs_stat<CB: Into<crate::FsCB<'static>>>(&self, path: &str, cb: CB) -> FsReqErrResult {
        self._fs_stat(path, cb)
    }

    /// Equivalent to stat(2).
    pub fn fs_stat_sync(&self, path: &str) -> Result<Stat, Box<dyn std::error::Error>> {
        self._fs_stat(path, ()).map(|mut req| {
            let stat = req.stat();
            req.destroy();
            return stat;
        })
    }

    /// Private implementation for fs_fstat()
    fn _fs_fstat<CB: Into<crate::FsCB<'static>>>(&self, file: File, cb: CB) -> FsReqResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let mut req = FsReq::new(cb)?;
        let result =
            crate::uvret(unsafe { uv_fs_fstat(self.into_inner(), req.inner(), file as _, uv_cb) });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to fstat(2).
    pub fn fs_fstat<CB: Into<crate::FsCB<'static>>>(&self, file: File, cb: CB) -> FsReqResult {
        self._fs_fstat(file, cb)
    }

    /// Equivalent to fstat(2).
    pub fn fs_fstat_sync(&self, file: File) -> crate::Result<Stat> {
        self._fs_fstat(file, ()).map(|mut req| {
            let stat = req.stat();
            req.destroy();
            return stat;
        })
    }

    /// Private implementation for fs_lstat
    fn _fs_lstat<CB: Into<crate::FsCB<'static>>>(&self, path: &str, cb: CB) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_lstat(self.into_inner(), req.inner(), path.as_ptr(), uv_cb)
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to lstat(2).
    pub fn fs_lstat<CB: Into<crate::FsCB<'static>>>(&self, path: &str, cb: CB) -> FsReqErrResult {
        self._fs_lstat(path, cb)
    }

    /// Equivalent to lstat(2).
    pub fn fs_lstat_sync(&self, path: &str) -> Result<Stat, Box<dyn std::error::Error>> {
        self._fs_lstat(path, ()).map(|mut req| {
            let stat = req.stat();
            req.destroy();
            return stat;
        })
    }

    /// Private implementation for fs_statfs()
    fn _fs_statfs<CB: Into<crate::FsCB<'static>>>(&self, path: &str, cb: CB) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_statfs(self.into_inner(), req.inner(), path.as_ptr(), uv_cb)
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
    pub fn fs_statfs<CB: Into<crate::FsCB<'static>>>(&self, path: &str, cb: CB) -> FsReqErrResult {
        self._fs_statfs(path, cb)
    }

    /// Equivalent to statfs(2). On success, FsReq::statfs() will return a StatFs
    ///
    /// Note: Any fields in the resulting StatFs that are not supported by the underlying operating
    /// system are set to zero.
    pub fn fs_statfs_sync(&self, path: &str) -> Result<StatFs, Box<dyn std::error::Error>> {
        self._fs_statfs(path, ()).and_then(|mut req| {
            let statfs = req.statfs();
            req.destroy();
            statfs.ok_or_else(|| Box::new(crate::Error::EINVAL) as _)
        })
    }

    /// Private implementation for fs_rename()
    fn _fs_rename<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        new_path: &str,
        cb: CB,
    ) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let new_path = CString::new(new_path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_rename(
                self.into_inner(),
                req.inner(),
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
    pub fn fs_rename<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        new_path: &str,
        cb: CB,
    ) -> FsReqErrResult {
        self._fs_rename(path, new_path, cb)
    }

    /// Equivalent to rename(2).
    pub fn fs_rename_sync(&self, path: &str, new_path: &str) -> SyncErrResult {
        self._fs_rename(path, new_path, ())
            .and_then(destroy_req_return_boxed_result)
    }

    /// Private implementation for fs_fsync()
    fn _fs_fsync<CB: Into<crate::FsCB<'static>>>(&self, file: File, cb: CB) -> FsReqResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let mut req = FsReq::new(cb)?;
        let result =
            crate::uvret(unsafe { uv_fs_fsync(self.into_inner(), req.inner(), file as _, uv_cb) });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to fsync(2).
    ///
    /// Note: For AIX, uv_fs_fsync returns UV_EBADF on file descriptors referencing non regular
    /// files.
    pub fn fs_fsync<CB: Into<crate::FsCB<'static>>>(&self, file: File, cb: CB) -> FsReqResult {
        self._fs_fsync(file, cb)
    }

    /// Equivalent to fsync(2).
    ///
    /// Note: For AIX, uv_fs_fsync returns UV_EBADF on file descriptors referencing non regular
    /// files.
    pub fn fs_fsync_sync(&self, file: File) -> SyncResult {
        self._fs_fsync(file, ()).and_then(destroy_req_return_result)
    }

    /// Private implementation for fs_fdatasync()
    fn _fs_fdatasync<CB: Into<crate::FsCB<'static>>>(&self, file: File, cb: CB) -> FsReqResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_fdatasync(self.into_inner(), req.inner(), file as _, uv_cb)
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to fdatasync(2).
    pub fn fs_fdatasync<CB: Into<crate::FsCB<'static>>>(&self, file: File, cb: CB) -> FsReqResult {
        self._fs_fdatasync(file, cb)
    }

    /// Equivalent to fdatasync(2).
    pub fn fs_fdatasync_sync(&self, file: File) -> SyncResult {
        self._fs_fdatasync(file, ())
            .and_then(destroy_req_return_result)
    }

    /// Private implementation for fs_ftruncate()
    fn _fs_ftruncate<CB: Into<crate::FsCB<'static>>>(
        &self,
        file: File,
        offset: i64,
        cb: CB,
    ) -> FsReqResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_ftruncate(self.into_inner(), req.inner(), file as _, offset, uv_cb)
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to ftruncate(2).
    pub fn fs_ftruncate<CB: Into<crate::FsCB<'static>>>(
        &self,
        file: File,
        offset: i64,
        cb: CB,
    ) -> FsReqResult {
        self._fs_ftruncate(file, offset, cb)
    }

    /// Equivalent to ftruncate(2).
    pub fn fs_ftruncate_sync(&self, file: File, offset: i64) -> SyncResult {
        self._fs_ftruncate(file, offset, ())
            .and_then(destroy_req_return_result)
    }

    /// Private implementation for fs_copyfile()
    fn _fs_copyfile<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        new_path: &str,
        flags: FsCopyFlags,
        cb: CB,
    ) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let new_path = CString::new(new_path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_copyfile(
                self.into_inner(),
                req.inner(),
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
    pub fn fs_copyfile<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        new_path: &str,
        flags: FsCopyFlags,
        cb: CB,
    ) -> FsReqErrResult {
        self._fs_copyfile(path, new_path, flags, cb)
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
        self._fs_copyfile(path, new_path, flags, ())
            .and_then(destroy_req_return_boxed_result)
    }

    /// Private implementation for fs_sendfile
    fn _fs_sendfile<CB: Into<crate::FsCB<'static>>>(
        &self,
        out_file: File,
        in_file: File,
        offset: i64,
        len: usize,
        cb: CB,
    ) -> FsReqResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_sendfile(
                self.into_inner(),
                req.inner(),
                out_file as _,
                in_file as _,
                offset,
                len as _,
                uv_cb,
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Limited equivalent to sendfile(2).
    pub fn fs_sendfile<CB: Into<crate::FsCB<'static>>>(
        &self,
        out_file: File,
        in_file: File,
        offset: i64,
        len: usize,
        cb: CB,
    ) -> FsReqResult {
        self._fs_sendfile(out_file, in_file, offset, len, cb)
    }

    /// Limited equivalent to sendfile(2).
    pub fn fs_sendfile_sync(
        &self,
        out_file: File,
        in_file: File,
        offset: i64,
        len: usize,
    ) -> SyncResult {
        self._fs_sendfile(out_file, in_file, offset, len, ())
            .and_then(destroy_req_return_result)
    }

    /// Private implementation for fs_access()
    fn _fs_access<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        mode: FsAccessFlags,
        cb: CB,
    ) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_access(
                self.into_inner(),
                req.inner(),
                path.as_ptr(),
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

    /// Equivalent to access(2) on Unix. Windows uses GetFileAttributesW().
    pub fn fs_access<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        mode: FsAccessFlags,
        cb: CB,
    ) -> FsReqErrResult {
        self._fs_access(path, mode, cb)
    }

    /// Equivalent to access(2) on Unix. Windows uses GetFileAttributesW().
    pub fn fs_access_sync(&self, path: &str, mode: FsAccessFlags) -> SyncErrResult {
        self._fs_access(path, mode, ())
            .and_then(destroy_req_return_boxed_result)
    }

    /// Private implementation for fs_chmod()
    fn _fs_chmod<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        mode: FsModeFlags,
        cb: CB,
    ) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_chmod(
                self.into_inner(),
                req.inner(),
                path.as_ptr(),
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

    /// Equivalent to chmod(2).
    pub fn fs_chmod<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        mode: FsModeFlags,
        cb: CB,
    ) -> FsReqErrResult {
        self._fs_chmod(path, mode, cb)
    }

    /// Equivalent to chmod(2).
    pub fn fs_chmod_sync(&self, path: &str, mode: FsModeFlags) -> SyncErrResult {
        self._fs_chmod(path, mode, ())
            .and_then(destroy_req_return_boxed_result)
    }

    /// Private implementation for fs_fchomd()
    fn _fs_fchmod<CB: Into<crate::FsCB<'static>>>(
        &self,
        file: File,
        mode: FsModeFlags,
        cb: CB,
    ) -> FsReqResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_fchmod(
                self.into_inner(),
                req.inner(),
                file as _,
                mode.bits(),
                uv_cb,
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to fchmod(2).
    pub fn fs_fchmod<CB: Into<crate::FsCB<'static>>>(
        &self,
        file: File,
        mode: FsModeFlags,
        cb: CB,
    ) -> FsReqResult {
        self._fs_fchmod(file, mode, cb)
    }

    /// Equivalent to fchmod(2).
    pub fn fs_fchmod_sync(&self, file: File, mode: FsModeFlags) -> SyncResult {
        self._fs_fchmod(file, mode, ())
            .and_then(destroy_req_return_result)
    }

    fn _fs_utime<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        atime: f64,
        mtime: f64,
        cb: CB,
    ) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_utime(
                self.into_inner(),
                req.inner(),
                path.as_ptr(),
                atime,
                mtime,
                uv_cb,
            )
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to utime(2).
    ///
    /// Note: AIX: This function only works for AIX 7.1 and newer. It can still be called on older
    /// versions but will return ENOSYS.
    pub fn fs_utime<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        atime: f64,
        mtime: f64,
        cb: CB,
    ) -> FsReqErrResult {
        self._fs_utime(path, atime, mtime, cb)
    }

    /// Equivalent to utime(2).
    ///
    /// Note: AIX: This function only works for AIX 7.1 and newer. It can still be called on older
    /// versions but will return ENOSYS.
    pub fn fs_utime_sync(&self, path: &str, atime: f64, mtime: f64) -> SyncErrResult {
        self._fs_utime(path, atime, mtime, ())
            .and_then(destroy_req_return_boxed_result)
    }

    /// Private implementation for fs_futime()
    fn _fs_futime<CB: Into<crate::FsCB<'static>>>(
        &self,
        file: File,
        atime: f64,
        mtime: f64,
        cb: CB,
    ) -> FsReqResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_futime(
                self.into_inner(),
                req.inner(),
                file as _,
                atime,
                mtime,
                uv_cb,
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to futimes(3) respectively.
    ///
    /// Note: AIX: This function only works for AIX 7.1 and newer. It can still be called on older
    /// versions but will return ENOSYS.
    pub fn fs_futime<CB: Into<crate::FsCB<'static>>>(
        &self,
        file: File,
        atime: f64,
        mtime: f64,
        cb: CB,
    ) -> FsReqResult {
        self._fs_futime(file, atime, mtime, cb)
    }

    /// Equivalent to futimes(3) respectively.
    ///
    /// Note: AIX: This function only works for AIX 7.1 and newer. It can still be called on older
    /// versions but will return ENOSYS.
    pub fn fs_futime_sync(&self, file: File, atime: f64, mtime: f64) -> SyncResult {
        self._fs_futime(file, atime, mtime, ())
            .and_then(destroy_req_return_result)
    }

    /// Private implementation for fs_link()
    fn _fs_link<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        new_path: &str,
        cb: CB,
    ) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let new_path = CString::new(new_path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_link(
                self.into_inner(),
                req.inner(),
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

    /// Equivalent to link(2).
    pub fn fs_link<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        new_path: &str,
        cb: CB,
    ) -> FsReqErrResult {
        self._fs_link(path, new_path, cb)
    }

    /// Equivalent to link(2).
    pub fn fs_link_sync(&self, path: &str, new_path: &str) -> SyncErrResult {
        self._fs_link(path, new_path, ())
            .and_then(destroy_req_return_boxed_result)
    }

    /// Private implementation for fs_symlink()
    fn _fs_symlink<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        new_path: &str,
        flags: FsSymlinkFlags,
        cb: CB,
    ) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let new_path = CString::new(new_path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_symlink(
                self.into_inner(),
                req.inner(),
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

    /// Equivalent to symlink(2).
    ///
    /// Note: On Windows the flags parameter can be specified to control how the symlink will be
    /// created:
    ///
    ///   * UV_FS_SYMLINK_DIR: indicates that path points to a directory.
    ///   * UV_FS_SYMLINK_JUNCTION: request that the symlink is created using junction points.
    pub fn fs_symlink<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        new_path: &str,
        flags: FsSymlinkFlags,
        cb: CB,
    ) -> FsReqErrResult {
        self._fs_symlink(path, new_path, flags, cb)
    }

    /// Equivalent to symlink(2).
    ///
    /// Note: On Windows the flags parameter can be specified to control how the symlink will be
    /// created:
    ///
    ///   * UV_FS_SYMLINK_DIR: indicates that path points to a directory.
    ///   * UV_FS_SYMLINK_JUNCTION: request that the symlink is created using junction points.
    pub fn fs_symlink_sync(
        &self,
        path: &str,
        new_path: &str,
        flags: FsSymlinkFlags,
    ) -> SyncErrResult {
        self._fs_symlink(path, new_path, flags, ())
            .and_then(destroy_req_return_boxed_result)
    }

    fn _fs_readlink<CB: Into<crate::FsCB<'static>>>(&self, path: &str, cb: CB) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_readlink(self.into_inner(), req.inner(), path.as_ptr(), uv_cb)
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to readlink(2). The path can be read from FsReq::real_path()
    pub fn fs_readlink<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        cb: CB,
    ) -> FsReqErrResult {
        self._fs_readlink(path, cb)
    }

    /// Equivalent to readlink(2).
    pub fn fs_readlink_sync(&self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        self._fs_readlink(path, ()).and_then(|mut req| {
            let path = req.real_path();
            req.destroy();
            path.ok_or_else(|| Box::new(crate::Error::EINVAL) as _)
        })
    }

    /// Private implementation for fs_realpath()
    fn _fs_realpath<CB: Into<crate::FsCB<'static>>>(&self, path: &str, cb: CB) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_realpath(self.into_inner(), req.inner(), path.as_ptr(), uv_cb)
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to realpath(3) on Unix. Windows uses GetFinalPathNameByHandle. The path can be
    /// read from FsReq::real_path()
    ///
    /// Warning: This function has certain platform-specific caveats that were discovered when used
    /// in Node.
    ///
    ///   * macOS and other BSDs: this function will fail with ELOOP if more than 32 symlinks are
    ///     found while resolving the given path. This limit is hardcoded and cannot be
    ///     sidestepped.
    ///   * Windows: while this function works in the common case, there are a number of corner
    ///     cases where it doesn’t:
    ///       * Paths in ramdisk volumes created by tools which sidestep the Volume Manager (such
    ///         as ImDisk) cannot be resolved.
    ///       * Inconsistent casing when using drive letters.
    ///       * Resolved path bypasses subst’d drives.
    ///
    /// While this function can still be used, it’s not recommended if scenarios such as the above
    /// need to be supported.
    ///
    /// Note: This function is not implemented on Windows XP and Windows Server 2003. On these
    /// systems, ENOSYS is returned.
    pub fn fs_realpath<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        cb: CB,
    ) -> FsReqErrResult {
        self._fs_realpath(path, cb)
    }

    /// Equivalent to realpath(3) on Unix. Windows uses GetFinalPathNameByHandle.
    ///
    /// Warning: This function has certain platform-specific caveats that were discovered when used
    /// in Node.
    ///
    ///   * macOS and other BSDs: this function will fail with ELOOP if more than 32 symlinks are
    ///     found while resolving the given path. This limit is hardcoded and cannot be
    ///     sidestepped.
    ///   * Windows: while this function works in the common case, there are a number of corner
    ///     cases where it doesn’t:
    ///       * Paths in ramdisk volumes created by tools which sidestep the Volume Manager (such
    ///         as ImDisk) cannot be resolved.
    ///       * Inconsistent casing when using drive letters.
    ///       * Resolved path bypasses subst’d drives.
    ///
    /// While this function can still be used, it’s not recommended if scenarios such as the above
    /// need to be supported.
    ///
    /// Note: This function is not implemented on Windows XP and Windows Server 2003. On these
    /// systems, ENOSYS is returned.
    pub fn fs_realpath_sync(&self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        self._fs_realpath(path, ()).and_then(|mut req| {
            let path = req.real_path();
            req.destroy();
            path.ok_or_else(|| Box::new(crate::Error::EINVAL) as _)
        })
    }

    /// Private implementation for fs_chown()
    fn _fs_chown<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        uid: Uid,
        gid: Gid,
        cb: CB,
    ) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_chown(
                self.into_inner(),
                req.inner(),
                path.as_ptr(),
                uid as _,
                gid as _,
                uv_cb,
            )
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to chown(2)
    ///
    /// Note: This functions are not implemented on Windows.
    pub fn fs_chown<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        uid: Uid,
        gid: Gid,
        cb: CB,
    ) -> FsReqErrResult {
        self._fs_chown(path, uid, gid, cb)
    }

    /// Equivalent to chown(2)
    ///
    /// Note: This functions are not implemented on Windows.
    pub fn fs_chown_sync(&self, path: &str, uid: Uid, gid: Gid) -> SyncErrResult {
        self._fs_chown(path, uid, gid, ())
            .and_then(destroy_req_return_boxed_result)
    }

    /// Private implementation for fs_fchown()
    fn _fs_fchown<CB: Into<crate::FsCB<'static>>>(
        &self,
        file: File,
        uid: Uid,
        gid: Gid,
        cb: CB,
    ) -> FsReqResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_fchown(
                self.into_inner(),
                req.inner(),
                file as _,
                uid as _,
                gid as _,
                uv_cb,
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to fchown(2)
    ///
    /// Note: This functions are not implemented on Windows.
    pub fn fs_fchown<CB: Into<crate::FsCB<'static>>>(
        &self,
        file: File,
        uid: Uid,
        gid: Gid,
        cb: CB,
    ) -> FsReqResult {
        self._fs_fchown(file, uid, gid, cb)
    }

    /// Equivalent to fchown(2)
    ///
    /// Note: This functions are not implemented on Windows.
    pub fn fs_fchown_sync(&self, file: File, uid: Uid, gid: Gid) -> SyncResult {
        self._fs_fchown(file, uid, gid, ())
            .and_then(destroy_req_return_result)
    }

    /// Private implementation for fs_lchown()
    fn _fs_lchown<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        uid: Uid,
        gid: Gid,
        cb: CB,
    ) -> FsReqErrResult {
        let cb = cb.into();
        let uv_cb = use_c_callback!(crate::uv_fs_cb, cb);
        let path = CString::new(path)?;
        let mut req = FsReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_fs_lchown(
                self.into_inner(),
                req.inner(),
                path.as_ptr(),
                uid as _,
                gid as _,
                uv_cb,
            )
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Equivalent to lchown(2)
    ///
    /// Note: This functions are not implemented on Windows.
    pub fn fs_lchown<CB: Into<crate::FsCB<'static>>>(
        &self,
        path: &str,
        uid: Uid,
        gid: Gid,
        cb: CB,
    ) -> FsReqErrResult {
        self._fs_lchown(path, uid, gid, cb)
    }

    /// Equivalent to lchown(2)
    ///
    /// Note: This functions are not implemented on Windows.
    pub fn fs_lchown_sync(&self, path: &str, uid: Uid, gid: Gid) -> SyncErrResult {
        self._fs_lchown(path, uid, gid, ())
            .and_then(destroy_req_return_boxed_result)
    }
}

/// An iterator using scandir to get a directory listing.
///
/// Note: Unlike scandir(3), this function does not return the “.” and “..” entries.
///
/// Note: On Linux, getting the type of an entry is only supported by some file systems (btrfs,
/// ext2, ext3 and ext4 at the time of this writing), check the getdents(2) man page.
pub struct ScandirIter {
    pub req: FsReq,
}

impl Iterator for ScandirIter {
    type Item = crate::Result<crate::Dirent>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut dirent: uv::uv_dirent_t = unsafe { std::mem::zeroed() };
        let result =
            crate::uvret(unsafe { uv_fs_scandir_next(self.req.inner(), &mut dirent as _) });
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
