use crate::{TryFromInner, TryIntoInner};
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

impl TryFromInner<uv_interface_address_t>  for InterfaceAddress {
    type Error = crate::Error;

    fn try_from_inner(addr: uv_interface_address_t) -> Result<InterfaceAddress, Self::Error> {
        let name = unsafe { CStr::from_ptr(addr.name) }.to_string_lossy().into_owned();
        let address = crate::build_socketaddr(uv_handle!(&addr.address))?;
        let netmask = crate::build_socketaddr(uv_handle!(&addr.netmask))?;
        let physical_address = [0u8; 6];
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

pub struct InterfaceAddressesIter {
    addresses: &'static [uv_interface_address_t],
    iter: std::slice::Iter<'static, uv_interface_address_t>,
}

impl InterfaceAddressesIter {
    pub(crate) fn new(addresses: *mut uv_interface_address_t, count: i32) -> InterfaceAddressesIter {
        let addresses = unsafe { std::slice::from_raw_parts(addresses, count as _) };
        let iter = addresses.iter();
        InterfaceAddressesIter { addresses, iter }
    }
}

impl Drop for InterfaceAddressesIter {
    fn drop(&mut self) {
        unsafe { uv_free_interface_addresses(self.addresses.as_mut_ptr() as _, self.addresses.len() as _) };
    }
}

impl Iterator for InterfaceAddressesIter {
    type Item = crate::Result<InterfaceAddress>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|addr| (*addr).try_into_inner())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(|addr| (*addr).try_into_inner())
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(|addr| (*addr).try_into_inner())
    }
}

impl ExactSizeIterator for InterfaceAddressesIter {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl DoubleEndedIterator for InterfaceAddressesIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|addr| (*addr).try_into_inner())
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).map(|addr| (*addr).try_into_inner())
    }
}

impl std::iter::FusedIterator for InterfaceAddressesIter {}
