
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    print!("ABCDEFGH");
    
    // Move to position and insert 3 characters
    print!("\x1b[1;4H");
    print!("\x1b[3@");
    print!("123");
    
    io::stdout().flush().unwrap();
}
