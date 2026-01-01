
use std::io::{self, Write};

fn main() {
    // Red text
    println!("\x1b[31mError: Something went wrong\x1b[0m");
    
    // Green text
    println!("\x1b[32mSuccess: Operation completed\x1b[0m");
    
    // Blue text
    println!("\x1b[34mInfo: Processing data\x1b[0m");
    
    // Yellow text
    println!("\x1b[33mWarning: Be careful\x1b[0m");
    
    // Cyan text
    println!("\x1b[36mDebug: Variable value = 42\x1b[0m");
    
    // Magenta text
    println!("\x1b[35mNote: Remember this\x1b[0m");
    
    io::stdout().flush().unwrap();
}
