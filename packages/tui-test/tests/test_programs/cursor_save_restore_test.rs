
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    print!("Start");
    print!("\x1b[s");           // Save cursor position
    
    print!("\x1b[5;5H");        // Move elsewhere
    print!("Middle");
    
    print!("\x1b[u");           // Restore cursor position
    print!("-End");
    
    io::stdout().flush().unwrap();
}
