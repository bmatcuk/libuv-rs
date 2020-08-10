use crate::{FromInner, IntoInner};
use uv::uv_dir_t;

/// Data type used for streaming directory iteration. Used by opendir(), readdir(), and closedir().
pub struct Dir {
    dir: *mut uv_dir_t,
    len: usize,
    capacity: usize,
}

impl Dir {
    /// Reserve space for directories. Use this before calling readdir() for the first time to
    /// allocate space.
    pub fn reserve(&mut self, size: usize) {
        self.free_entries();

        let mut v = std::mem::ManuallyDrop::new(Vec::<uv::uv_dirent_t>::with_capacity(size));
        unsafe {
            (*self.dir).dirents = v.as_mut_ptr();
            (*self.dir).nentries = v.capacity() as _;
        }
        self.capacity = v.capacity();
        self.len = v.len();
    }

    /// Deallocate the space for directories that was allocated with reserve()
    pub fn free_entries(&mut self) {
        unsafe {
            if !(*self.dir).dirents.is_null() {
                std::mem::drop(Vec::from_raw_parts(
                    (*self.dir).dirents,
                    self.len,
                    self.capacity,
                ));
                (*self.dir).dirents = std::ptr::null_mut();
                (*self.dir).nentries = 0;
                self.capacity = 0;
                self.len = 0;
            }
        }
    }

    /// The number of directory entries
    pub fn len(&self) -> usize {
        self.len
    }

    /// Used by readdir() to set the length of returned directory entries
    pub(crate) fn set_len(&mut self, len: usize) {
        self.len = len;
    }

    /// The maximum number of directory entries that can be retrieved per call to fs_readdir
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Create an iterator over the directory entries
    pub fn entries(&self) -> Vec<crate::Dirent> {
        let v = unsafe {
            std::mem::ManuallyDrop::new(Vec::<uv::uv_dirent_t>::from_raw_parts(
                (*self.dir).dirents,
                self.len,
                self.capacity,
            ))
        };
        v.iter()
            .map(|d| crate::Dirent::from_inner(d as *const uv::uv_dirent_t))
            .collect()
    }
}

impl FromInner<*mut uv_dir_t> for Dir {
    fn from_inner(dir: *mut uv_dir_t) -> Dir {
        Dir {
            dir,
            len: 0,
            capacity: 0,
        }
    }
}

impl IntoInner<*mut uv_dir_t> for &Dir {
    fn into_inner(self) -> *mut uv_dir_t {
        self.dir
    }
}
