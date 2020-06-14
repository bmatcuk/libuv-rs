extern crate libuv;
use libuv::prelude::*;

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

fn after_fib(i: usize) {
    println!("Done calculating {}th fibonacci", i);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut r#loop = Loop::default()?;

    for i in 0..FIB_UNTIL {
        r#loop.queue_work(move |_| fib(i), move |_, _| after_fib(i))?;
    }

    r#loop.run(RunMode::Default)?;

    Ok(())
}
