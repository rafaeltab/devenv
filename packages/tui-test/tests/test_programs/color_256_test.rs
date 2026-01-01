
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    // Use 256-color mode - color 196 is bright red
    print!("\x1b[38;5;196mBright Red\x1b[0m\n");
    
    // Color 46 is bright green
    print!("\x1b[38;5;46mBright Green\x1b[0m\n");
    
    io::stdout().flush().unwrap();
}
