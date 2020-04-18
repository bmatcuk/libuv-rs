extern crate libuv;
use libuv::interface_addresses;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let interfaces = interface_addresses()?;
    println!("Number of interfaces: {}", interfaces.len());
    for info in interfaces.iter() {
        println!("Name: {}", info.name);
        println!("Internal?: {}", if info.is_internal { "Yes" } else { "No" });
        println!("IP address: {}", info.address);
        println!("");
    }

    Ok(())
}
