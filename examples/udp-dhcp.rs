extern crate libuv;
use libuv::prelude::*;
use libuv::{Buf, ReadonlyBuf, UdpBindFlags, UdpHandle};
use std::net::{Ipv4Addr, SocketAddr};

fn alloc_buffer(_: Handle, suggested_size: usize) -> Option<Buf> {
    Buf::with_capacity(suggested_size).ok()
}

fn on_read(mut handle: UdpHandle, nread: libuv::Result<isize>, mut buf: ReadonlyBuf, addr: SocketAddr, flags: UdpBindFlags) {
    match nread {
        Ok(_) => {
        },
        Err(e) => {
            eprintln!("Read error {}", e);
            handle.close(None::<fn(Handle)>);
        }
    }

    buf.dealloc();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let mut recv_socket = r#loop.udp()?;
    let recv_addr = (Ipv4Addr::UNSPECIFIED, 68).into();
    recv_socket.bind(&recv_addr, UdpBindFlags::REUSEADDR)?;
    recv_socket.recv_start(Some(alloc_buffer), Some(on_read))?;

    let mut send_socket = r#loop.udp()?;
    let broadcast_addr = (Ipv4Addr::UNSPECIFIED, 0).into();
    send_socket.bind(&broadcast_addr, UdpBindFlags::empty())?;
    send_socket.set_broadcast(true)?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
