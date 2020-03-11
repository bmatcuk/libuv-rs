#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum DirentType {
    BLOCK,
    CHAR,
    DIR,
    FIFO,
    FILE,
    LINK,
    SOCKET,
    UNKNOWN,
}

impl crate::FromInner<uv::uv_dirent_type_t> for DirentType {
    fn from_inner(t: uv::uv_dirent_type_t) -> DirentType {
        match t {
            uv::uv_dirent_type_t_UV_DIRENT_BLOCK => DirentType::BLOCK,
            uv::uv_dirent_type_t_UV_DIRENT_CHAR => DirentType::CHAR,
            uv::uv_dirent_type_t_UV_DIRENT_DIR => DirentType::DIR,
            uv::uv_dirent_type_t_UV_DIRENT_FIFO => DirentType::FIFO,
            uv::uv_dirent_type_t_UV_DIRENT_FILE => DirentType::FILE,
            uv::uv_dirent_type_t_UV_DIRENT_LINK => DirentType::LINK,
            uv::uv_dirent_type_t_UV_DIRENT_SOCKET => DirentType::SOCKET,
            _ => DirentType::UNKNOWN,
        }
    }
}

impl crate::IntoInner<uv::uv_dirent_type_t> for &DirentType {
    fn into_inner(self) -> uv::uv_dirent_type_t {
        match self {
            DirentType::BLOCK => uv::uv_dirent_type_t_UV_DIRENT_BLOCK,
            DirentType::CHAR => uv::uv_dirent_type_t_UV_DIRENT_CHAR,
            DirentType::DIR => uv::uv_dirent_type_t_UV_DIRENT_DIR,
            DirentType::FIFO => uv::uv_dirent_type_t_UV_DIRENT_FIFO,
            DirentType::FILE => uv::uv_dirent_type_t_UV_DIRENT_FILE,
            DirentType::LINK => uv::uv_dirent_type_t_UV_DIRENT_LINK,
            DirentType::SOCKET => uv::uv_dirent_type_t_UV_DIRENT_SOCKET,
            _ => uv::uv_dirent_type_t_UV_DIRENT_UNKNOWN,
        }
    }
}
