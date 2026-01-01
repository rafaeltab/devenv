
use std::io::{self, Write};

fn main() {
    println!("First line of text");
    println!("Second line of text");
    println!("Third line of text");
    io::stdout().flush().unwrap();
    
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    // Clear screen
    print!("\x1b[2J\x1b[H");
    println!("Screen cleared!");
    io::stdout().flush().unwrap();
}
