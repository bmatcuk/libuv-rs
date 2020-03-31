#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FsType {
    ACCESS,
    CHMOD,
    CHOWN,
    CLOSE,
    CLOSEDIR,
    COPYFILE,
    CUSTOM,
    FCHMOD,
    FCHOWN,
    FDATASYNC,
    FSTAT,
    FSYNC,
    FTRUNCATE,
    FUTIME,
    LCHOWN,
    LINK,
    LSTAT,
    MKDIR,
    MKDTEMP,
    MKSTEMP,
    OPEN,
    OPENDIR,
    READ,
    READDIR,
    READLINK,
    REALPATH,
    RENAME,
    RMDIR,
    SCANDIR,
    SENDFILE,
    STAT,
    STATFS,
    SYMLINK,
    UNLINK,
    UTIME,
    WRITE,
    UNKNOWN,
}

impl crate::FromInner<uv::uv_fs_type> for FsType {
    fn from_inner(t: uv::uv_fs_type) -> FsType {
        match t {
            uv::uv_fs_type_UV_FS_ACCESS => FsType::ACCESS,
            uv::uv_fs_type_UV_FS_CHMOD => FsType::CHMOD,
            uv::uv_fs_type_UV_FS_CHOWN => FsType::CHOWN,
            uv::uv_fs_type_UV_FS_CLOSE => FsType::CLOSE,
            uv::uv_fs_type_UV_FS_CLOSEDIR => FsType::CLOSEDIR,
            uv::uv_fs_type_UV_FS_COPYFILE => FsType::COPYFILE,
            uv::uv_fs_type_UV_FS_CUSTOM => FsType::CUSTOM,
            uv::uv_fs_type_UV_FS_FCHMOD => FsType::FCHMOD,
            uv::uv_fs_type_UV_FS_FCHOWN => FsType::FCHOWN,
            uv::uv_fs_type_UV_FS_FDATASYNC => FsType::FDATASYNC,
            uv::uv_fs_type_UV_FS_FSTAT => FsType::FSTAT,
            uv::uv_fs_type_UV_FS_FSYNC => FsType::FSYNC,
            uv::uv_fs_type_UV_FS_FTRUNCATE => FsType::FTRUNCATE,
            uv::uv_fs_type_UV_FS_FUTIME => FsType::FUTIME,
            uv::uv_fs_type_UV_FS_LCHOWN => FsType::LCHOWN,
            uv::uv_fs_type_UV_FS_LINK => FsType::LINK,
            uv::uv_fs_type_UV_FS_LSTAT => FsType::LSTAT,
            uv::uv_fs_type_UV_FS_MKDIR => FsType::MKDIR,
            uv::uv_fs_type_UV_FS_MKDTEMP => FsType::MKDTEMP,
            uv::uv_fs_type_UV_FS_MKSTEMP => FsType::MKSTEMP,
            uv::uv_fs_type_UV_FS_OPEN => FsType::OPEN,
            uv::uv_fs_type_UV_FS_OPENDIR => FsType::OPENDIR,
            uv::uv_fs_type_UV_FS_READ => FsType::READ,
            uv::uv_fs_type_UV_FS_READDIR => FsType::READDIR,
            uv::uv_fs_type_UV_FS_READLINK => FsType::READLINK,
            uv::uv_fs_type_UV_FS_REALPATH => FsType::REALPATH,
            uv::uv_fs_type_UV_FS_RENAME => FsType::RENAME,
            uv::uv_fs_type_UV_FS_RMDIR => FsType::RMDIR,
            uv::uv_fs_type_UV_FS_SCANDIR => FsType::SCANDIR,
            uv::uv_fs_type_UV_FS_SENDFILE => FsType::SENDFILE,
            uv::uv_fs_type_UV_FS_STAT => FsType::STAT,
            uv::uv_fs_type_UV_FS_STATFS => FsType::STATFS,
            uv::uv_fs_type_UV_FS_SYMLINK => FsType::SYMLINK,
            uv::uv_fs_type_UV_FS_UNLINK => FsType::UNLINK,
            uv::uv_fs_type_UV_FS_UTIME => FsType::UTIME,
            uv::uv_fs_type_UV_FS_WRITE => FsType::WRITE,
            _ => FsType::UNKNOWN,
        }
    }
}

impl crate::IntoInner<uv::uv_fs_type> for &FsType {
    fn into_inner(self) -> uv::uv_fs_type {
        match self {
            FsType::ACCESS => uv::uv_fs_type_UV_FS_ACCESS,
            FsType::CHMOD => uv::uv_fs_type_UV_FS_CHMOD,
            FsType::CHOWN => uv::uv_fs_type_UV_FS_CHOWN,
            FsType::CLOSE => uv::uv_fs_type_UV_FS_CLOSE,
            FsType::CLOSEDIR => uv::uv_fs_type_UV_FS_CLOSEDIR,
            FsType::COPYFILE => uv::uv_fs_type_UV_FS_COPYFILE,
            FsType::CUSTOM => uv::uv_fs_type_UV_FS_CUSTOM,
            FsType::FCHMOD => uv::uv_fs_type_UV_FS_FCHMOD,
            FsType::FCHOWN => uv::uv_fs_type_UV_FS_FCHOWN,
            FsType::FDATASYNC => uv::uv_fs_type_UV_FS_FDATASYNC,
            FsType::FSTAT => uv::uv_fs_type_UV_FS_FSTAT,
            FsType::FSYNC => uv::uv_fs_type_UV_FS_FSYNC,
            FsType::FTRUNCATE => uv::uv_fs_type_UV_FS_FTRUNCATE,
            FsType::FUTIME => uv::uv_fs_type_UV_FS_FUTIME,
            FsType::LCHOWN => uv::uv_fs_type_UV_FS_LCHOWN,
            FsType::LINK => uv::uv_fs_type_UV_FS_LINK,
            FsType::LSTAT => uv::uv_fs_type_UV_FS_LSTAT,
            FsType::MKDIR => uv::uv_fs_type_UV_FS_MKDIR,
            FsType::MKDTEMP => uv::uv_fs_type_UV_FS_MKDTEMP,
            FsType::MKSTEMP => uv::uv_fs_type_UV_FS_MKSTEMP,
            FsType::OPEN => uv::uv_fs_type_UV_FS_OPEN,
            FsType::OPENDIR => uv::uv_fs_type_UV_FS_OPENDIR,
            FsType::READ => uv::uv_fs_type_UV_FS_READ,
            FsType::READDIR => uv::uv_fs_type_UV_FS_READDIR,
            FsType::READLINK => uv::uv_fs_type_UV_FS_READLINK,
            FsType::REALPATH => uv::uv_fs_type_UV_FS_REALPATH,
            FsType::RENAME => uv::uv_fs_type_UV_FS_RENAME,
            FsType::RMDIR => uv::uv_fs_type_UV_FS_RMDIR,
            FsType::SCANDIR => uv::uv_fs_type_UV_FS_SCANDIR,
            FsType::SENDFILE => uv::uv_fs_type_UV_FS_SENDFILE,
            FsType::STAT => uv::uv_fs_type_UV_FS_STAT,
            FsType::STATFS => uv::uv_fs_type_UV_FS_STATFS,
            FsType::SYMLINK => uv::uv_fs_type_UV_FS_SYMLINK,
            FsType::UNLINK => uv::uv_fs_type_UV_FS_UNLINK,
            FsType::UTIME => uv::uv_fs_type_UV_FS_UTIME,
            FsType::WRITE => uv::uv_fs_type_UV_FS_WRITE,
            _ => uv::uv_fs_type_UV_FS_UNKNOWN,
        }
    }
}
