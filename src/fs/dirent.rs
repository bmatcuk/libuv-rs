include!("./dirent_types.inc.rs");

use crate::{FromInner, IntoInner};
use uv::uv_dirent_t;

/// Cross platform (reduced) equivalent of struct dirent. Used in scandir_next().
pub struct Dirent {
    pub name: String,
    pub r#type: DirentType,
}

impl FromInner<*const uv_dirent_t> for Dirent {
    fn from_inner(dirent: *const uv_dirent_t) -> Dirent {
        let name = unsafe { std::ffi::CStr::from_ptr((*dirent).name) }
            .to_string_lossy()
            .into_owned();
        Dirent {
            name,
            r#type: unsafe { (*dirent).type_.into_inner() },
        }
    }
}
