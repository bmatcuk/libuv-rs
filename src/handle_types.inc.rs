#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum HandleType {
    ASYNC,
    CHECK,
    FILE,
    FS_EVENT,
    FS_POLL,
    HANDLE,
    IDLE,
    NAMED_PIPE,
    POLL,
    PREPARE,
    PROCESS,
    SIGNAL,
    STREAM,
    TCP,
    TIMER,
    TTY,
    UDP,
    UNKNOWN,
}

impl From<uv::uv_handle_type> for HandleType {
    fn from(t: uv::uv_handle_type) -> HandleType {
        match t {
            uv::uv_handle_type_UV_ASYNC => HandleType::ASYNC,
            uv::uv_handle_type_UV_CHECK => HandleType::CHECK,
            uv::uv_handle_type_UV_FILE => HandleType::FILE,
            uv::uv_handle_type_UV_FS_EVENT => HandleType::FS_EVENT,
            uv::uv_handle_type_UV_FS_POLL => HandleType::FS_POLL,
            uv::uv_handle_type_UV_HANDLE => HandleType::HANDLE,
            uv::uv_handle_type_UV_IDLE => HandleType::IDLE,
            uv::uv_handle_type_UV_NAMED_PIPE => HandleType::NAMED_PIPE,
            uv::uv_handle_type_UV_POLL => HandleType::POLL,
            uv::uv_handle_type_UV_PREPARE => HandleType::PREPARE,
            uv::uv_handle_type_UV_PROCESS => HandleType::PROCESS,
            uv::uv_handle_type_UV_SIGNAL => HandleType::SIGNAL,
            uv::uv_handle_type_UV_STREAM => HandleType::STREAM,
            uv::uv_handle_type_UV_TCP => HandleType::TCP,
            uv::uv_handle_type_UV_TIMER => HandleType::TIMER,
            uv::uv_handle_type_UV_TTY => HandleType::TTY,
            uv::uv_handle_type_UV_UDP => HandleType::UDP,
            _ => HandleType::UNKNOWN,
        }
    }
}

impl Into<uv_handle_type> for &HandleType {
    fn into(self) -> uv::uv_handle_type {
        match self {
            HandleType::ASYNC => uv::uv_handle_type_UV_ASYNC,
            HandleType::CHECK => uv::uv_handle_type_UV_CHECK,
            HandleType::FILE => uv::uv_handle_type_UV_FILE,
            HandleType::FS_EVENT => uv::uv_handle_type_UV_FS_EVENT,
            HandleType::FS_POLL => uv::uv_handle_type_UV_FS_POLL,
            HandleType::HANDLE => uv::uv_handle_type_UV_HANDLE,
            HandleType::IDLE => uv::uv_handle_type_UV_IDLE,
            HandleType::NAMED_PIPE => uv::uv_handle_type_UV_NAMED_PIPE,
            HandleType::POLL => uv::uv_handle_type_UV_POLL,
            HandleType::PREPARE => uv::uv_handle_type_UV_PREPARE,
            HandleType::PROCESS => uv::uv_handle_type_UV_PROCESS,
            HandleType::SIGNAL => uv::uv_handle_type_UV_SIGNAL,
            HandleType::STREAM => uv::uv_handle_type_UV_STREAM,
            HandleType::TCP => uv::uv_handle_type_UV_TCP,
            HandleType::TIMER => uv::uv_handle_type_UV_TIMER,
            HandleType::TTY => uv::uv_handle_type_UV_TTY,
            HandleType::UDP => uv::uv_handle_type_UV_UDP,
            _ => uv::uv_handle_type_UV_UNKNOWN_HANDLE,
        }
    }
}
