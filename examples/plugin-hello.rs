//! This is part of the plugin example. See plugin.rs for documentation

#[no_mangle]
pub extern "C" fn initialize() {
    println!("Hello, World!");
}
