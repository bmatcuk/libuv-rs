extern crate libuv;
use libuv::prelude::*;
use libuv::{AddrInfo, GetAddrInfoReq};
use libuv_sys2::{AF_INET, SOCK_STREAM, IPPROTO_TCP};

fn on_resolved(req: GetAddrInfoReq, status: libuv::Result<i32>, res: Vec<AddrInfo>) {
    if let Err(e) = status {
        eprintln!("getaddrinfo callback error {}", e);
        return;
    }

    if let Some(info) = res.get(0) {
        if let Some(addr) = info.addr {
            println!("{}", addr);

            let mut socket = req.r#loop().tcp();
            match socket {
                Ok(socket) => {
                    if let Err(e) = socket.connect(&addr, Some(on_connect)) {
                        eprintln!("error connecting socket: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("cannot create socket: {}", e);
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let hints = AddrInfo {
        flags: 0,
        family: AF_INET,
        socktype: SOCK_STREAM,
        protocol: IPPROTO_TCP,
        canonical_name: None,
        addr: None,
    };
    r#loop.getaddrinfo(Some("irc.freenode.net"), Some("6667"), Some(hints), on_resolved)?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
