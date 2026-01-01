
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    print!("A\tB\tC\tD");
    
    io::stdout().flush().unwrap();
}
