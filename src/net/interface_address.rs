use crate::{FromInner, IntoInner, TryFromInner, TryIntoInner};
use std::ffi::CStr;
use std::net::SocketAddr;
use uv::{uv_free_interface_addresses, uv_interface_address_t};

/// Data type for interface addresses.
pub struct InterfaceAddress {
    pub name: String,
    pub physical_address: [u8; 6],
    pub is_internal: bool,
    pub address: SocketAddr,
    pub netmask: SocketAddr,
}

impl TryFromInner<*mut uv_interface_address_t>  for InterfaceAddress {
    type Error = crate::Error;

    fn try_from_inner(addr: *mut uv_interface_address_t) -> Result<InterfaceAddress, Self::Error> {
        let name = unsafe { CStr::from_ptr((*addr).name) }.to_string_lossy().into_owned();
        let address = crate::build_socketaddr(uv_handle!(&(*addr).address))?;
        let netmask = crate::build_socketaddr(uv_handle!(&(*addr).netmask))?;
        let physical_address = [0u8; 6];
        for (i, b) in (*addr).phys_addr.iter().enumerate() {
            physical_address[i] = *b as _;
        }

        Ok(InterfaceAddress {
            name,
            physical_address,
            is_internal: (*addr).is_internal != 0,
            address,
            netmask,
        })
    }
}
