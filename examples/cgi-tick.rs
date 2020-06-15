//! This is part of the cgi example - see cgi.rs for documentation

fn main() {
    for _ in 0..10 {
        println!("tick");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    println!("BOOM!");
}
