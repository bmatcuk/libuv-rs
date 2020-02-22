#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum ReqType {
    CONNECT,
    FS,
    GETADDRINFO,
    GETNAMEINFO,
    RANDOM,
    REQ,
    SHUTDOWN,
    UDP_SEND,
    WORK,
    WRITE,
    UNKNOWN,
}

impl From<uv::uv_req_type> for ReqType {
    fn from(t: uv::uv_req_type) -> ReqType {
        match t {
            uv::uv_req_type_UV_CONNECT => ReqType::CONNECT,
            uv::uv_req_type_UV_FS => ReqType::FS,
            uv::uv_req_type_UV_GETADDRINFO => ReqType::GETADDRINFO,
            uv::uv_req_type_UV_GETNAMEINFO => ReqType::GETNAMEINFO,
            uv::uv_req_type_UV_RANDOM => ReqType::RANDOM,
            uv::uv_req_type_UV_REQ => ReqType::REQ,
            uv::uv_req_type_UV_SHUTDOWN => ReqType::SHUTDOWN,
            uv::uv_req_type_UV_UDP_SEND => ReqType::UDP_SEND,
            uv::uv_req_type_UV_WORK => ReqType::WORK,
            uv::uv_req_type_UV_WRITE => ReqType::WRITE,
            _ => ReqType::UNKNOWN,
        }
    }
}

impl Into<uv::uv_req_type> for &ReqType {
    fn into(self) -> uv::uv_req_type {
        match self {
            ReqType::CONNECT => uv::uv_req_type_UV_CONNECT,
            ReqType::FS => uv::uv_req_type_UV_FS,
            ReqType::GETADDRINFO => uv::uv_req_type_UV_GETADDRINFO,
            ReqType::GETNAMEINFO => uv::uv_req_type_UV_GETNAMEINFO,
            ReqType::RANDOM => uv::uv_req_type_UV_RANDOM,
            ReqType::REQ => uv::uv_req_type_UV_REQ,
            ReqType::SHUTDOWN => uv::uv_req_type_UV_SHUTDOWN,
            ReqType::UDP_SEND => uv::uv_req_type_UV_UDP_SEND,
            ReqType::WORK => uv::uv_req_type_UV_WORK,
            ReqType::WRITE => uv::uv_req_type_UV_WRITE,
            _ => uv::uv_req_type_UV_UNKNOWN_REQ,
        }
    }
}
