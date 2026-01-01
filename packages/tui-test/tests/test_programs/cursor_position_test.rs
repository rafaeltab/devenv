
use std::io::{self, Write};

fn main() {
    // Clear screen first
    print!("\x1b[2J\x1b[H");
    
    // Write at position (1, 1) - top left
    print!("\x1b[1;1HTop-Left");
    
    // Write at position (5, 10)
    print!("\x1b[5;10HMiddle");
    
    // Write at position (1, 20)
    print!("\x1b[1;20HTop-Right");
    
    io::stdout().flush().unwrap();
}
