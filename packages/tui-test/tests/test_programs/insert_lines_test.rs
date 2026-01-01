
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    println!("Line A");
    println!("Line B");
    println!("Line C");
    
    // Go to line 2 and insert a line
    print!("\x1b[2;1H");
    print!("\x1b[L");
    print!("INSERTED");
    
    io::stdout().flush().unwrap();
}
