//! Run:
//!
//! ```bash
//! cargo run --example signal
//! ```
//!
//! Then send SIGUSR1 and SIGUSR2 to the process:
//!
//! ```bash
//! kill -USR1 PID
//! kill -USR2 PID
//! ```

extern crate libuv;
use libuv::prelude::*;
use libuv::{getpid, SignalHandle};
use libuv_sys2::{SIGUSR1, SIGUSR2};
use std::thread;

fn signal_handler(mut handle: SignalHandle, signum: i32) {
    println!("Signal received {}", signum);
    if let Err(e) = handle.stop() {
        eprintln!("Error stopping signal: {}", e);
    }
}

// Two signals in one loop
fn thread_worker1() {
    fn worker() -> Result<(), Box<dyn std::error::Error>> {
        let mut r#loop = Loop::new()?;

        let mut sig1 = r#loop.signal()?;
        sig1.start(signal_handler, SIGUSR1 as _)?;

        let mut sig2 = r#loop.signal()?;
        sig2.start(signal_handler, SIGUSR2 as _)?;

        r#loop.run(RunMode::Default)?;

        // close signals
        sig1.close(());
        sig2.close(());
        r#loop.run(RunMode::Default)?;

        Ok(())
    }

    if let Err(e) = worker() {
        eprintln!("Error in thread_worker1: {}", e);
    }
}

// Two signal handlers, each in its own loop
fn thread_worker2() {
    fn worker() -> Result<(), Box<dyn std::error::Error>> {
        let mut loop1 = Loop::new()?;
        let mut loop2 = Loop::new()?;

        let mut sig1 = loop1.signal()?;
        sig1.start(signal_handler, SIGUSR1 as _)?;

        let mut sig2 = loop2.signal()?;
        sig2.start(signal_handler, SIGUSR2 as _)?;

        loop {
            let ret1 = loop1.run(RunMode::NoWait)?;
            let ret2 = loop2.run(RunMode::NoWait)?;
            if ret1 == 0 && ret2 == 0 {
                break;
            }
        }

        // close signals
        sig1.close(());
        sig2.close(());
        loop1.run(RunMode::Default)?;
        loop2.run(RunMode::Default)?;

        Ok(())
    }

    if let Err(e) = worker() {
        eprintln!("Error in thread_worker2: {}", e);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("PID {}", getpid());

    let thread1 = thread::spawn(thread_worker1);
    let thread2 = thread::spawn(thread_worker2);

    thread1.join().unwrap();
    thread2.join().unwrap();

    Ok(())
}
