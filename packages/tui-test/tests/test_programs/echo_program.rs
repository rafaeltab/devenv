
use std::io::{self, Write, BufRead};

fn main() {
    println!("Enter your name:");
    io::stdout().flush().unwrap();
    
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();
    
    let name = line.trim();
    println!("Hello, {}!", name);
    println!("Your name has {} characters.", name.len());
}
