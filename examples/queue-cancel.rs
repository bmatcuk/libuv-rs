extern crate libuv;
use libuv::prelude::*;
use libuv::{SignalHandle, WorkReq};
use libuv_sys2::SIGINT;

const FIB_UNTIL: usize = 25;

fn fib(i: usize) {
    fn fib_(i: u64) -> u64 {
        if i == 0 || i == 1 {
            return 1;
        }
        fib_(i - 1) + fib_(i - 2)
    }

    if rand::random() {
        std::thread::sleep(std::time::Duration::from_secs(1));
    } else {
        std::thread::sleep(std::time::Duration::from_secs(3));
    }

    let fib = fib_(i as _);
    println!("{}th fibonacci is {}", i, fib);
}

fn after_fib(i: usize, status: libuv::Result<u32>) {
    match status {
        Ok(_) => println!("Done calculating {}th fibonacci", i),
        Err(libuv::Error::ECANCELED) => eprintln!("Calculation of {} cancelled.", i),
        Err(e) => eprintln!("Error {}", e),
    }
}

fn signal_handler(mut sig: SignalHandle, reqs: &mut [WorkReq]) {
    println!("Signal received");
    for req in reqs.iter_mut() {
        // intentionally ignoring errors - any work requests that have already finished will
        // "error" and we don't care about that
        let _ = req.cancel();
    }

    if let Err(e) = sig.stop() {
        eprintln!("Could not stop signal handler {}", e);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    let mut reqs = (0..FIB_UNTIL)
        .map(|i| {
            r#loop.queue_work(
                Some(move |_| fib(i)),
                Some(move |_, status| after_fib(i, status)),
            )
        })
        .collect::<libuv::Result<Vec<WorkReq>>>()?;

    let mut sig = r#loop.signal()?;
    sig.start(Some(move |handle, _| signal_handler(handle, &mut reqs)), SIGINT as _)?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
