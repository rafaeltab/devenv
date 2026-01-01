
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    // Write a full line
    print!("This is a very long line of text");
    
    // Move cursor back and erase to end of line
    print!("\x1b[1;10H");  // Move to column 10
    print!("\x1b[K");       // Erase from cursor to end of line
    print!("SHORT");        // Write new text
    
    io::stdout().flush().unwrap();
}
