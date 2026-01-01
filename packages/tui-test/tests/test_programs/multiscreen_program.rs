
use std::io::{self, Write, BufRead};

fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    io::stdout().flush().unwrap();
}

fn main() {
    // Screen 1: Welcome
    clear_screen();
    println!("Welcome to Multi-Screen App");
    println!("\nPress Enter to continue...");
    io::stdout().flush().unwrap();
    
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();
    
    // Screen 2: Input
    clear_screen();
    println!("Please enter a number (1-3):");
    io::stdout().flush().unwrap();
    
    line.clear();
    stdin.lock().read_line(&mut line).unwrap();
    let choice: u8 = line.trim().parse().unwrap_or(1);
    
    // Screen 3: Result
    clear_screen();
    println!("You chose option {}", choice);
    
    match choice {
        1 => println!("\x1b[32mGood choice!\x1b[0m"),
        2 => println!("\x1b[33mInteresting choice!\x1b[0m"),
        3 => println!("\x1b[34mBold choice!\x1b[0m"),
        _ => {}
    }
    
    println!("\nThank you for using this app!");
    io::stdout().flush().unwrap();
}
