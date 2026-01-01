
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    // Set scrolling region (lines 5-10)
    print!("\x1b[5;10r");
    
    // Fill the screen
    for i in 1..=15 {
        print!("\x1b[{};1H", i);
        print!("Line {}", i);
    }
    
    io::stdout().flush().unwrap();
}
