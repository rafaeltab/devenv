
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    // True color: RGB(255, 100, 50) - orange-ish
    print!("\x1b[38;2;255;100;50mOrange Text\x1b[0m\n");
    
    // True color: RGB(100, 200, 255) - light blue
    print!("\x1b[38;2;100;200;255mLight Blue\x1b[0m\n");
    
    io::stdout().flush().unwrap();
}
