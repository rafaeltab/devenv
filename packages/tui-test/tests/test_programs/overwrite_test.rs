
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    print!("XXXXXXXXXX");
    
    // Move back to start and overwrite
    print!("\x1b[1;1H");
    print!("HELLO");
    
    io::stdout().flush().unwrap();
}
