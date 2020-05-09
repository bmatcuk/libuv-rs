extern crate libuv;
use libuv::prelude::*;
use libuv::{
    exepath, ProcessHandle, ProcessOptions, StdioContainer, StdioFlags, StdioType, TcpBindFlags,
    TcpHandle,
};
use std::net::Ipv4Addr;
use std::path::PathBuf;

fn invoke_cgi_script(client: TcpHandle) -> Result<(), Box<dyn std::error::Error>> {
    let mut path: PathBuf = exepath()?.into();
    if cfg!(windows) {
        path.set_file_name("cgi-tick.exe");
    } else {
        path.set_file_name("cgi-tick");
    }

    let child_stdio = [
        Default::default(),
        StdioContainer {
            flags: StdioFlags::INHERIT_STREAM,
            data: StdioType::Stream(client.to_stream()),
        },
        Default::default(),
    ];

    let path = path.to_string_lossy().into_owned();
    let args: [&str; 1] = [&path];
    let mut options = ProcessOptions::new(&args);
    options.stdio = &child_stdio;

    let mut client_clone = client.clone();
    options.exit_cb = (move |mut handle: ProcessHandle, exit_status: i64, term_signal: i32| {
        println!(
            "Process exited with status {}, signal {}",
            exit_status, term_signal
        );
        handle.close(());
        client_clone.close(());
    })
    .into();

    client.get_loop().spawn_process(options)?;

    Ok(())
}

fn on_new_connection(mut server: StreamHandle, status: libuv::Result<u32>) {
    if let Err(e) = status {
        eprintln!("New connection error: {}", e);
        return;
    }

    if let Ok(mut client) = server.get_loop().tcp() {
        match server.accept(&mut client.to_stream()) {
            Ok(_) => {
                if let Err(e) = invoke_cgi_script(client) {
                    eprintln!("Error invoking CGI script: {}", e);
                }
            }
            Err(_) => {
                client.close(());
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let mut server = r#loop.tcp()?;
    let addr = (Ipv4Addr::UNSPECIFIED, 7000).into();
    server.bind(&addr, TcpBindFlags::empty())?;
    server.listen(128, on_new_connection)?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
