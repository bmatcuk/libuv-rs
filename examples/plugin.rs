//! You must build plugin-hello.rs first:
//!
//! ```bash
//! cargo build --example plugin-hello
//! ```
//!
//! Then run:
//!
//! ```bash
//! cargo run --example plugin -- target/debug/examples/libplugin-hello.dylib
//! ```

extern crate libuv;
use libuv::DLib;

type InitializeFunc = unsafe extern "C" fn();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut plugins = std::env::args();
    if plugins.len() <= 1 {
        if let Some(prog) = plugins.nth(0) {
            eprintln!("Usare: {} [plugin1] [plugin2]", prog);
        };
        return Ok(());
    }

    for plugin in plugins.skip(1) {
        println!("Loading {}", plugin);
        match DLib::open(&plugin) {
            Ok(lib) => match lib.sym::<InitializeFunc>("initialize") {
                Ok(func) => unsafe { (*func)() },
                Err(e) => eprintln!("dlsym error: {}", e),
            },
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
