extern crate libuv;
use libuv::{Loop, RunMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let r#loop = Loop::new()?;

    println!("Now quitting.");
    r#loop.run(RunMode.Default)?;

    r#loop.close()?;
}
