
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    print!("Line 1\n");
    print!("Line 2\n");
    print!("Line 3\n");
    
    // Move up 2 lines
    print!("\x1b[2A");
    // Move forward 7 characters
    print!("\x1b[7C");
    print!("X");
    
    io::stdout().flush().unwrap();
    
    // Small delay to ensure output is transmitted
    std::thread::sleep(std::time::Duration::from_millis(50));
}
