
use std::io::{self, Write, BufRead};

fn main() {
    let items = vec!["Option A", "Option B", "Option C"];
    let mut selected = 0;

    // Initial display
    print!("\x1b[2J\x1b[H");
    println!("Select an option (type number 0-2, then press Enter):\n");
    
    for (i, item) in items.iter().enumerate() {
        if i == selected {
            println!("\x1b[33m> {}\x1b[0m", item);
        } else {
            println!("  {}", item);
        }
    }
    
    println!("\nCurrent selection: {}", selected);
    io::stdout().flush().unwrap();

    // Read selection
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();
    
    if let Ok(choice) = line.trim().parse::<usize>() {
        if choice < items.len() {
            selected = choice;
        }
    }
    
    // Show final selection
    print!("\x1b[2J\x1b[H");
    println!("You selected: {}", items[selected]);
    io::stdout().flush().unwrap();
}
