
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    // Set foreground red, background blue
    print!("\x1b[31;44m");
    print!("Normal");
    
    // Reverse video
    print!("\x1b[7m");
    print!("Reversed");
    
    io::stdout().flush().unwrap();
}
