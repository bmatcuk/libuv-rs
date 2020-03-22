use std::ffi::CStr;
use uv::{uv_if_indextoiid, uv_if_indextoname, uv_interface_addresses, UV_IF_NAMESIZE};

/// IPv6-capable implementation of if_indextoname(3).
///
/// On Unix, the returned interface name can be used directly as an interface identifier in scoped
/// IPv6 addresses, e.g. fe80::abc:def1:2345%en0.
///
/// On Windows, the returned interface cannot be used as an interface identifier, as Windows uses
/// numerical interface identifiers, e.g. fe80::abc:def1:2345%5.
///
/// To get an interface identifier in a cross-platform compatible way, use if_indextoiid().
pub fn if_indextoname(ifindex: u32) -> crate::Result<String> {
    let mut size = UV_IF_NAMESIZE as usize;
    let mut buf: Vec<std::os::raw::c_uchar> = Vec::with_capacity(size);
    crate::uvret(unsafe { uv_if_indextoname(ifindex, buf.as_mut_ptr() as _, &mut size as _) }).map(
        |_| {
            // size is the length of the string, *not* including the null
            unsafe { buf.set_len(size + 1) };
            unsafe { CStr::from_bytes_with_nul_unchecked(&buf) }
                .to_string_lossy()
                .into_owned()
        },
    )
}

/// Retrieves a network interface identifier suitable for use in an IPv6 scoped address. On
/// Windows, returns the numeric ifindex as a string. On all other platforms, if_indextoname() is
/// called.
///
/// See uv_if_indextoname for further details.
pub fn if_indexto_iid(ifindex: u32) -> crate::Result<String> {
    let mut size = UV_IF_NAMESIZE as usize;
    let mut buf: Vec<std::os::raw::c_uchar> = Vec::with_capacity(size);
    crate::uvret(unsafe { uv_if_indextoiid(ifindex, buf.as_mut_ptr() as _, &mut size as _) }).map(
        |_| {
            // size is the length of the string, *not* including the null
            unsafe { buf.set_len(size + 1) };
            unsafe { CStr::from_bytes_with_nul_unchecked(&buf) }
                .to_string_lossy()
                .into_owned()
        },
    )
}

/// Gets address information about the network interfaces on the system.
pub fn interface_addresses() -> crate::Result<crate::InterfaceAddressesIter> {
    let mut addresses: *mut uv::uv_interface_address_t = std::mem::zeroed();
    let mut count: std::os::raw::c_int = 0;
    crate::uvret(unsafe { uv_interface_addresses(&mut addresses as _, &mut count as _) })?;
    Ok(crate::InterfaceAddressesIter::new(addresses, count))
}
