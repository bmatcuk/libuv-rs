#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Error {
    E2BIG,
    EACCES,
    EADDRINUSE,
    EADDRNOTAVAIL,
    EAFNOSUPPORT,
    EAGAIN,
    EAI_ADDRFAMILY,
    EAI_AGAIN,
    EAI_BADFLAGS,
    EAI_BADHINTS,
    EAI_CANCELED,
    EAI_FAIL,
    EAI_FAMILY,
    EAI_MEMORY,
    EAI_NODATA,
    EAI_NONAME,
    EAI_OVERFLOW,
    EAI_PROTOCOL,
    EAI_SERVICE,
    EAI_SOCKTYPE,
    EALREADY,
    EBADF,
    EBUSY,
    ECANCELED,
    ECHARSET,
    ECONNABORTED,
    ECONNREFUSED,
    ECONNRESET,
    EDESTADDRREQ,
    EEXIST,
    EFAULT,
    EFBIG,
    EFTYPE,
    EHOSTDOWN,
    EHOSTUNREACH,
    EILSEQ,
    EINTR,
    EINVAL,
    EIO,
    EISCONN,
    EISDIR,
    ELOOP,
    EMFILE,
    EMLINK,
    EMSGSIZE,
    ENAMETOOLONG,
    ENETDOWN,
    ENETUNREACH,
    ENFILE,
    ENOBUFS,
    ENODEV,
    ENOENT,
    ENOMEM,
    ENONET,
    ENOPROTOOPT,
    ENOSPC,
    ENOSYS,
    ENOTCONN,
    ENOTDIR,
    ENOTEMPTY,
    ENOTSOCK,
    ENOTSUP,
    ENOTTY,
    ENXIO,
    EOF,
    EPERM,
    EPIPE,
    EPROTO,
    EPROTONOSUPPORT,
    EPROTOTYPE,
    ERANGE,
    EREMOTEIO,
    EROFS,
    ERRNO_MAX,
    ESHUTDOWN,
    ESPIPE,
    ESRCH,
    ETIMEDOUT,
    ETXTBSY,
    EXDEV,
    UNKNOWN,
}

impl From<uv::uv_errno_t> for Error {
    fn from(code: uv::uv_errno_t) -> Error {
        match code {
            uv::uv_errno_t_UV_E2BIG => Error::E2BIG,
            uv::uv_errno_t_UV_EACCES => Error::EACCES,
            uv::uv_errno_t_UV_EADDRINUSE => Error::EADDRINUSE,
            uv::uv_errno_t_UV_EADDRNOTAVAIL => Error::EADDRNOTAVAIL,
            uv::uv_errno_t_UV_EAFNOSUPPORT => Error::EAFNOSUPPORT,
            uv::uv_errno_t_UV_EAGAIN => Error::EAGAIN,
            uv::uv_errno_t_UV_EAI_ADDRFAMILY => Error::EAI_ADDRFAMILY,
            uv::uv_errno_t_UV_EAI_AGAIN => Error::EAI_AGAIN,
            uv::uv_errno_t_UV_EAI_BADFLAGS => Error::EAI_BADFLAGS,
            uv::uv_errno_t_UV_EAI_BADHINTS => Error::EAI_BADHINTS,
            uv::uv_errno_t_UV_EAI_CANCELED => Error::EAI_CANCELED,
            uv::uv_errno_t_UV_EAI_FAIL => Error::EAI_FAIL,
            uv::uv_errno_t_UV_EAI_FAMILY => Error::EAI_FAMILY,
            uv::uv_errno_t_UV_EAI_MEMORY => Error::EAI_MEMORY,
            uv::uv_errno_t_UV_EAI_NODATA => Error::EAI_NODATA,
            uv::uv_errno_t_UV_EAI_NONAME => Error::EAI_NONAME,
            uv::uv_errno_t_UV_EAI_OVERFLOW => Error::EAI_OVERFLOW,
            uv::uv_errno_t_UV_EAI_PROTOCOL => Error::EAI_PROTOCOL,
            uv::uv_errno_t_UV_EAI_SERVICE => Error::EAI_SERVICE,
            uv::uv_errno_t_UV_EAI_SOCKTYPE => Error::EAI_SOCKTYPE,
            uv::uv_errno_t_UV_EALREADY => Error::EALREADY,
            uv::uv_errno_t_UV_EBADF => Error::EBADF,
            uv::uv_errno_t_UV_EBUSY => Error::EBUSY,
            uv::uv_errno_t_UV_ECANCELED => Error::ECANCELED,
            uv::uv_errno_t_UV_ECHARSET => Error::ECHARSET,
            uv::uv_errno_t_UV_ECONNABORTED => Error::ECONNABORTED,
            uv::uv_errno_t_UV_ECONNREFUSED => Error::ECONNREFUSED,
            uv::uv_errno_t_UV_ECONNRESET => Error::ECONNRESET,
            uv::uv_errno_t_UV_EDESTADDRREQ => Error::EDESTADDRREQ,
            uv::uv_errno_t_UV_EEXIST => Error::EEXIST,
            uv::uv_errno_t_UV_EFAULT => Error::EFAULT,
            uv::uv_errno_t_UV_EFBIG => Error::EFBIG,
            uv::uv_errno_t_UV_EFTYPE => Error::EFTYPE,
            uv::uv_errno_t_UV_EHOSTDOWN => Error::EHOSTDOWN,
            uv::uv_errno_t_UV_EHOSTUNREACH => Error::EHOSTUNREACH,
            uv::uv_errno_t_UV_EILSEQ => Error::EILSEQ,
            uv::uv_errno_t_UV_EINTR => Error::EINTR,
            uv::uv_errno_t_UV_EINVAL => Error::EINVAL,
            uv::uv_errno_t_UV_EIO => Error::EIO,
            uv::uv_errno_t_UV_EISCONN => Error::EISCONN,
            uv::uv_errno_t_UV_EISDIR => Error::EISDIR,
            uv::uv_errno_t_UV_ELOOP => Error::ELOOP,
            uv::uv_errno_t_UV_EMFILE => Error::EMFILE,
            uv::uv_errno_t_UV_EMLINK => Error::EMLINK,
            uv::uv_errno_t_UV_EMSGSIZE => Error::EMSGSIZE,
            uv::uv_errno_t_UV_ENAMETOOLONG => Error::ENAMETOOLONG,
            uv::uv_errno_t_UV_ENETDOWN => Error::ENETDOWN,
            uv::uv_errno_t_UV_ENETUNREACH => Error::ENETUNREACH,
            uv::uv_errno_t_UV_ENFILE => Error::ENFILE,
            uv::uv_errno_t_UV_ENOBUFS => Error::ENOBUFS,
            uv::uv_errno_t_UV_ENODEV => Error::ENODEV,
            uv::uv_errno_t_UV_ENOENT => Error::ENOENT,
            uv::uv_errno_t_UV_ENOMEM => Error::ENOMEM,
            uv::uv_errno_t_UV_ENONET => Error::ENONET,
            uv::uv_errno_t_UV_ENOPROTOOPT => Error::ENOPROTOOPT,
            uv::uv_errno_t_UV_ENOSPC => Error::ENOSPC,
            uv::uv_errno_t_UV_ENOSYS => Error::ENOSYS,
            uv::uv_errno_t_UV_ENOTCONN => Error::ENOTCONN,
            uv::uv_errno_t_UV_ENOTDIR => Error::ENOTDIR,
            uv::uv_errno_t_UV_ENOTEMPTY => Error::ENOTEMPTY,
            uv::uv_errno_t_UV_ENOTSOCK => Error::ENOTSOCK,
            uv::uv_errno_t_UV_ENOTSUP => Error::ENOTSUP,
            uv::uv_errno_t_UV_ENOTTY => Error::ENOTTY,
            uv::uv_errno_t_UV_ENXIO => Error::ENXIO,
            uv::uv_errno_t_UV_EOF => Error::EOF,
            uv::uv_errno_t_UV_EPERM => Error::EPERM,
            uv::uv_errno_t_UV_EPIPE => Error::EPIPE,
            uv::uv_errno_t_UV_EPROTO => Error::EPROTO,
            uv::uv_errno_t_UV_EPROTONOSUPPORT => Error::EPROTONOSUPPORT,
            uv::uv_errno_t_UV_EPROTOTYPE => Error::EPROTOTYPE,
            uv::uv_errno_t_UV_ERANGE => Error::ERANGE,
            uv::uv_errno_t_UV_EREMOTEIO => Error::EREMOTEIO,
            uv::uv_errno_t_UV_EROFS => Error::EROFS,
            uv::uv_errno_t_UV_ERRNO_MAX => Error::ERRNO_MAX,
            uv::uv_errno_t_UV_ESHUTDOWN => Error::ESHUTDOWN,
            uv::uv_errno_t_UV_ESPIPE => Error::ESPIPE,
            uv::uv_errno_t_UV_ESRCH => Error::ESRCH,
            uv::uv_errno_t_UV_ETIMEDOUT => Error::ETIMEDOUT,
            uv::uv_errno_t_UV_ETXTBSY => Error::ETXTBSY,
            uv::uv_errno_t_UV_EXDEV => Error::EXDEV,
            uv::uv_errno_t_UV_UNKNOWN => Error::UNKNOWN,
            _ => Error::UNKNOWN,
        }
    }
}

impl Error {
    fn code(&self) -> uv::uv_errno_t {
        match self {
            Error::E2BIG => uv::uv_errno_t_UV_E2BIG,
            Error::EACCES => uv::uv_errno_t_UV_EACCES,
            Error::EADDRINUSE => uv::uv_errno_t_UV_EADDRINUSE,
            Error::EADDRNOTAVAIL => uv::uv_errno_t_UV_EADDRNOTAVAIL,
            Error::EAFNOSUPPORT => uv::uv_errno_t_UV_EAFNOSUPPORT,
            Error::EAGAIN => uv::uv_errno_t_UV_EAGAIN,
            Error::EAI_ADDRFAMILY => uv::uv_errno_t_UV_EAI_ADDRFAMILY,
            Error::EAI_AGAIN => uv::uv_errno_t_UV_EAI_AGAIN,
            Error::EAI_BADFLAGS => uv::uv_errno_t_UV_EAI_BADFLAGS,
            Error::EAI_BADHINTS => uv::uv_errno_t_UV_EAI_BADHINTS,
            Error::EAI_CANCELED => uv::uv_errno_t_UV_EAI_CANCELED,
            Error::EAI_FAIL => uv::uv_errno_t_UV_EAI_FAIL,
            Error::EAI_FAMILY => uv::uv_errno_t_UV_EAI_FAMILY,
            Error::EAI_MEMORY => uv::uv_errno_t_UV_EAI_MEMORY,
            Error::EAI_NODATA => uv::uv_errno_t_UV_EAI_NODATA,
            Error::EAI_NONAME => uv::uv_errno_t_UV_EAI_NONAME,
            Error::EAI_OVERFLOW => uv::uv_errno_t_UV_EAI_OVERFLOW,
            Error::EAI_PROTOCOL => uv::uv_errno_t_UV_EAI_PROTOCOL,
            Error::EAI_SERVICE => uv::uv_errno_t_UV_EAI_SERVICE,
            Error::EAI_SOCKTYPE => uv::uv_errno_t_UV_EAI_SOCKTYPE,
            Error::EALREADY => uv::uv_errno_t_UV_EALREADY,
            Error::EBADF => uv::uv_errno_t_UV_EBADF,
            Error::EBUSY => uv::uv_errno_t_UV_EBUSY,
            Error::ECANCELED => uv::uv_errno_t_UV_ECANCELED,
            Error::ECHARSET => uv::uv_errno_t_UV_ECHARSET,
            Error::ECONNABORTED => uv::uv_errno_t_UV_ECONNABORTED,
            Error::ECONNREFUSED => uv::uv_errno_t_UV_ECONNREFUSED,
            Error::ECONNRESET => uv::uv_errno_t_UV_ECONNRESET,
            Error::EDESTADDRREQ => uv::uv_errno_t_UV_EDESTADDRREQ,
            Error::EEXIST => uv::uv_errno_t_UV_EEXIST,
            Error::EFAULT => uv::uv_errno_t_UV_EFAULT,
            Error::EFBIG => uv::uv_errno_t_UV_EFBIG,
            Error::EFTYPE => uv::uv_errno_t_UV_EFTYPE,
            Error::EHOSTDOWN => uv::uv_errno_t_UV_EHOSTDOWN,
            Error::EHOSTUNREACH => uv::uv_errno_t_UV_EHOSTUNREACH,
            Error::EILSEQ => uv::uv_errno_t_UV_EILSEQ,
            Error::EINTR => uv::uv_errno_t_UV_EINTR,
            Error::EINVAL => uv::uv_errno_t_UV_EINVAL,
            Error::EIO => uv::uv_errno_t_UV_EIO,
            Error::EISCONN => uv::uv_errno_t_UV_EISCONN,
            Error::EISDIR => uv::uv_errno_t_UV_EISDIR,
            Error::ELOOP => uv::uv_errno_t_UV_ELOOP,
            Error::EMFILE => uv::uv_errno_t_UV_EMFILE,
            Error::EMLINK => uv::uv_errno_t_UV_EMLINK,
            Error::EMSGSIZE => uv::uv_errno_t_UV_EMSGSIZE,
            Error::ENAMETOOLONG => uv::uv_errno_t_UV_ENAMETOOLONG,
            Error::ENETDOWN => uv::uv_errno_t_UV_ENETDOWN,
            Error::ENETUNREACH => uv::uv_errno_t_UV_ENETUNREACH,
            Error::ENFILE => uv::uv_errno_t_UV_ENFILE,
            Error::ENOBUFS => uv::uv_errno_t_UV_ENOBUFS,
            Error::ENODEV => uv::uv_errno_t_UV_ENODEV,
            Error::ENOENT => uv::uv_errno_t_UV_ENOENT,
            Error::ENOMEM => uv::uv_errno_t_UV_ENOMEM,
            Error::ENONET => uv::uv_errno_t_UV_ENONET,
            Error::ENOPROTOOPT => uv::uv_errno_t_UV_ENOPROTOOPT,
            Error::ENOSPC => uv::uv_errno_t_UV_ENOSPC,
            Error::ENOSYS => uv::uv_errno_t_UV_ENOSYS,
            Error::ENOTCONN => uv::uv_errno_t_UV_ENOTCONN,
            Error::ENOTDIR => uv::uv_errno_t_UV_ENOTDIR,
            Error::ENOTEMPTY => uv::uv_errno_t_UV_ENOTEMPTY,
            Error::ENOTSOCK => uv::uv_errno_t_UV_ENOTSOCK,
            Error::ENOTSUP => uv::uv_errno_t_UV_ENOTSUP,
            Error::ENOTTY => uv::uv_errno_t_UV_ENOTTY,
            Error::ENXIO => uv::uv_errno_t_UV_ENXIO,
            Error::EOF => uv::uv_errno_t_UV_EOF,
            Error::EPERM => uv::uv_errno_t_UV_EPERM,
            Error::EPIPE => uv::uv_errno_t_UV_EPIPE,
            Error::EPROTO => uv::uv_errno_t_UV_EPROTO,
            Error::EPROTONOSUPPORT => uv::uv_errno_t_UV_EPROTONOSUPPORT,
            Error::EPROTOTYPE => uv::uv_errno_t_UV_EPROTOTYPE,
            Error::ERANGE => uv::uv_errno_t_UV_ERANGE,
            Error::EREMOTEIO => uv::uv_errno_t_UV_EREMOTEIO,
            Error::EROFS => uv::uv_errno_t_UV_EROFS,
            Error::ERRNO_MAX => uv::uv_errno_t_UV_ERRNO_MAX,
            Error::ESHUTDOWN => uv::uv_errno_t_UV_ESHUTDOWN,
            Error::ESPIPE => uv::uv_errno_t_UV_ESPIPE,
            Error::ESRCH => uv::uv_errno_t_UV_ESRCH,
            Error::ETIMEDOUT => uv::uv_errno_t_UV_ETIMEDOUT,
            Error::ETXTBSY => uv::uv_errno_t_UV_ETXTBSY,
            Error::EXDEV => uv::uv_errno_t_UV_EXDEV,
            Error::UNKNOWN => uv::uv_errno_t_UV_UNKNOWN,
        }
    }
}
