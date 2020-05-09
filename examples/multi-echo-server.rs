extern crate libuv;
use libuv::prelude::*;
use libuv::{
    cpu_info, exepath, Buf, PipeHandle, ProcessHandle, ProcessOptions, StdioContainer, StdioFlags,
    StdioType, TcpBindFlags,
};
use std::net::Ipv4Addr;
use std::path::PathBuf;

struct Workers {
    workers: Vec<PipeHandle>,
    next: usize,
}

impl Workers {
    fn new(capacity: usize) -> Workers {
        Workers {
            workers: Vec::with_capacity(capacity),
            next: 0,
        }
    }

    fn add(&mut self, worker: PipeHandle) {
        self.workers.push(worker);
    }

    fn next(&mut self) -> &mut PipeHandle {
        let next = (self.next + 1) % self.workers.len();
        let ret = unsafe { self.workers.get_unchecked_mut(self.next) };
        self.next = next;
        return ret;
    }
}

fn close_process_handle(mut handle: ProcessHandle, exit_status: i64, term_signal: i32) {
    println!(
        "Process exited with status {}, signal {}",
        exit_status, term_signal
    );
    handle.close(());
}

fn on_new_connection(mut server: StreamHandle, status: libuv::Result<u32>, workers: &mut Workers) {
    if let Err(e) = status {
        eprintln!("Error with new connection: {}", e);
        return;
    }

    if let Ok(mut client) = server.get_loop().tcp() {
        match server.accept(&mut client.to_stream()) {
            Ok(_) => {
                if let Ok(buf) = Buf::new("a") {
                    let worker = workers.next();
                    if let Err(e) = worker.write2(&client.to_stream(), &[buf], ()) {
                        eprintln!("Could not write to worker: {}", e);
                    }
                }
            }
            Err(_) => {
                client.close(());
            }
        }
    }
}

fn setup_workers(r#loop: &mut Loop) -> Result<Workers, Box<dyn std::error::Error>> {
    let mut path: PathBuf = exepath()?.into();
    if cfg!(windows) {
        path.set_file_name("multi-echo-server-worker.exe");
    } else {
        path.set_file_name("multi-echo-server-worker");
    }

    let path = path.to_string_lossy().into_owned();
    let args: [&str; 1] = [&path];

    let info = cpu_info()?;
    let mut workers = Workers::new(info.len());
    for _ in 0..info.len() {
        let pipe = r#loop.pipe(true)?;
        workers.add(pipe);

        let child_stdio = [
            StdioContainer {
                flags: StdioFlags::CREATE_PIPE | StdioFlags::READABLE_PIPE,
                data: StdioType::Stream(pipe.to_stream()),
            },
            Default::default(),
            StdioContainer {
                flags: StdioFlags::INHERIT_FD,
                data: StdioType::Fd(2),
            },
        ];

        let mut options = ProcessOptions::new(&args);
        options.exit_cb = close_process_handle.into();
        options.stdio = &child_stdio;

        let process = r#loop.spawn_process(options)?;
        println!("Started worker {}", process.pid());
    }

    Ok(workers)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let mut workers = setup_workers(&mut r#loop)?;

    let mut server = r#loop.tcp()?;
    let addr = (Ipv4Addr::UNSPECIFIED, 7000).into();
    server.bind(&addr, TcpBindFlags::empty())?;
    server.listen(128, move |server, status| on_new_connection(server, status, &mut workers))?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
