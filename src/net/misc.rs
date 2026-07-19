use crate::{TryFromInner, TryIntoInner};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::CStr;
use core::net::SocketAddr;
use uv::{
    uv_free_interface_addresses, uv_if_indextoiid, uv_if_indextoname, uv_interface_address_t,
    uv_interface_addresses, UV_IF_NAMESIZE,
};

/// Data type for interface addresses.
pub struct InterfaceAddress {
    pub name: String,
    pub physical_address: [u8; 6],
    pub is_internal: bool,
    pub address: SocketAddr,
    pub netmask: SocketAddr,
}

impl TryFromInner<&uv_interface_address_t> for InterfaceAddress {
    type Error = Box<dyn core::error::Error>;

    fn try_from_inner(addr: &uv_interface_address_t) -> Result<InterfaceAddress, Self::Error> {
        let name = unsafe { CStr::from_ptr(addr.name) }
            .to_string_lossy()
            .into_owned();
        let address = crate::build_socketaddr(uv_handle!(&addr.address))?;
        let netmask = crate::build_socketaddr(uv_handle!(&addr.netmask))?;
        let mut physical_address = [0u8; 6];
        for (i, b) in addr.phys_addr.iter().enumerate() {
            physical_address[i] = *b as _;
        }

        Ok(InterfaceAddress {
            name,
            physical_address,
            is_internal: addr.is_internal != 0,
            address,
            netmask,
        })
    }
}

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
    let mut buf: Vec<core::ffi::c_uchar> = Vec::with_capacity(size as _);
    crate::uvret(unsafe { uv_if_indextoname(ifindex, buf.as_mut_ptr() as _, &mut size as _) }).map(
        |_| {
            // size is the length of the string, *not* including the null
            unsafe { buf.set_len((size as usize) + 1) };
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
    let mut buf: Vec<core::ffi::c_uchar> = Vec::with_capacity(size as _);
    crate::uvret(unsafe { uv_if_indextoiid(ifindex, buf.as_mut_ptr() as _, &mut size as _) }).map(
        |_| {
            // size is the length of the string, *not* including the null
            unsafe { buf.set_len((size as usize) + 1) };
            unsafe { CStr::from_bytes_with_nul_unchecked(&buf) }
                .to_string_lossy()
                .into_owned()
        },
    )
}

/// Gets address information about the network interfaces on the system.
pub fn interface_addresses() -> Result<Vec<InterfaceAddress>, Box<dyn core::error::Error>> {
    let mut addresses: *mut uv::uv_interface_address_t = unsafe { core::mem::zeroed() };
    let mut count: core::ffi::c_int = 0;
    crate::uvret(unsafe { uv_interface_addresses(&mut addresses as _, &mut count as _) })?;

    let result = unsafe { core::slice::from_raw_parts(addresses, count as _) }
        .iter()
        .map(|addr| addr.try_into_inner())
        .collect();
    unsafe { uv_free_interface_addresses(addresses, count as _) };
    result
}
