extern crate libuv;
use libuv::prelude::*;
use libuv::{AddrInfo, Buf, ConnectReq, GetAddrInfoReq, ReadonlyBuf};
use libuv_sys2::{AF_INET, IPPROTO_TCP, SOCK_STREAM};

fn alloc_buffer(_: Handle, suggested_size: usize) -> Option<Buf> {
    Buf::with_capacity(suggested_size).ok()
}

fn on_read(mut client: StreamHandle, nread: libuv::Result<usize>, mut buf: ReadonlyBuf) {
    match nread {
        Ok(len) => {
            if let Err(e) = client.read_stop() {
                eprintln!("cannot stop read {}", e);
            }

            match buf.to_str(len) {
                Ok(s) => println!("{}", s),
                Err(e) => eprintln!("couldn't convert to string {}", e),
            }
        }
        Err(e) => {
            if e != libuv::Error::EOF {
                eprintln!("Read error {}", e);
            }
            client.close(());
        }
    }

    buf.dealloc();
}

fn on_connect(req: ConnectReq, status: libuv::Result<u32>) {
    match status {
        Ok(_) => {
            if let Err(e) = req.handle().read_start(alloc_buffer, on_read) {
                eprintln!("error starting read {}", e)
            }
        }
        Err(e) => eprintln!("connect failed error {}", e),
    }
}

fn on_resolved(req: GetAddrInfoReq, status: libuv::Result<u32>, res: Vec<AddrInfo>) {
    if let Err(e) = status {
        eprintln!("getaddrinfo callback error {}", e);
        return;
    }

    for info in res {
        if let Some(addr) = info.addr {
            println!("{}", addr);

            let socket = req.r#loop().tcp();
            match socket {
                Ok(mut socket) => {
                    if let Err(e) = socket.connect(&addr, on_connect) {
                        eprintln!("error connecting socket: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("cannot create socket: {}", e);
                }
            }

            return;
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
    r#loop.getaddrinfo(
        Some("irc.freenode.net"),
        Some("6667"),
        Some(hints),
        on_resolved,
    )?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
