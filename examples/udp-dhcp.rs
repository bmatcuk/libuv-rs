extern crate libuv;
use libuv::prelude::*;
use libuv::{Buf, ReadonlyBuf, UdpBindFlags, UdpHandle};
use rand::prelude::*;
use std::net::{Ipv4Addr, SocketAddr};

fn alloc_buffer(_: Handle, suggested_size: usize) -> Option<Buf> {
    Buf::with_capacity(suggested_size).ok()
}

fn on_read(
    mut handle: UdpHandle,
    nread: libuv::Result<usize>,
    mut buf: ReadonlyBuf,
    addr: SocketAddr,
    _flags: UdpBindFlags,
) {
    match nread {
        Ok(_) => {
            let ip = &buf[16..20];
            println!("Recv from {}", addr);
            println!("Offered IP {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
        }
        Err(e) => {
            eprintln!("Read error {}", e);
        }
    }

    if let Err(e) = handle.recv_stop() {
        eprintln!("Cannot stop recv: {}", e);
    }

    buf.dealloc();
}

fn make_discover_msg() -> Result<Buf, Box<dyn std::error::Error>> {
    let mut buf = [0u8; 256];

    buf[0] = 0x1; // BOOTREQUEST
    buf[1] = 0x1; // HTYPE ethernet
    buf[2] = 0x6; // HLEN
    buf[3] = 0x0; // HOPS

    // XID
    rand::thread_rng().fill_bytes(&mut buf[4..8]);

    buf[8] = 0x0; // SECS
    buf[10] = 0x80; // FLAGS

    // CIADDR 12-15 all zeros
    // YIADDR 16-19 all zeros
    // SIADDR 20-23 all zeros
    // GIADDR 24-27 all zeros

    // CHADDR 28-43 is the MAC address, use your own
    buf[28] = 0xe4;
    buf[29] = 0xce;
    buf[30] = 0x8f;
    buf[31] = 0x13;
    buf[32] = 0xf6;
    buf[33] = 0xd4;

    // SNAME 64 bytes zero
    // FILE 128 bytes zero

    // OPTIONS
    // - magic cookie
    buf[236] = 0x63;
    buf[237] = 0x82;
    buf[238] = 0x53;
    buf[239] = 0x63;

    // DHCP Message type - DHCPDISCOVER
    buf[240] = 0x35;
    buf[241] = 0x01;
    buf[242] = 0x01;

    // DHCP Parameter request list
    buf[243] = 0x37;
    buf[244] = 0x04;
    buf[245] = 0x01;
    buf[246] = 0x03;
    buf[247] = 0x0f;
    buf[248] = 0x06;

    // End mark
    buf[249] = 0xff;

    Buf::new_from_bytes(&buf)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let mut recv_socket = r#loop.udp()?;
    let recv_addr = (Ipv4Addr::UNSPECIFIED, 68).into();
    recv_socket.bind(&recv_addr, UdpBindFlags::REUSEADDR)?;
    recv_socket.recv_start(alloc_buffer, on_read)?;

    let mut send_socket = r#loop.udp()?;
    let broadcast_addr = (Ipv4Addr::UNSPECIFIED, 0).into();
    send_socket.bind(&broadcast_addr, UdpBindFlags::empty())?;
    send_socket.set_broadcast(true)?;

    let mut discover_msg = make_discover_msg()?;
    let send_addr = (Ipv4Addr::BROADCAST, 67).into();
    send_socket.send(Some(&send_addr), &[discover_msg], move |_, status| {
        if let Err(e) = status {
            eprintln!("Send error {}", e);
        }
        discover_msg.destroy();
    })?;

    r#loop.run(RunMode::Default)?;

    recv_socket.close(());
    send_socket.close(());

    Ok(())
}
