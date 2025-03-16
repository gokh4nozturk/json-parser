use json_parser::parse_json;
use std::env;
use std::fs;
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = env::args().collect();

    let json_str = if args.len() > 1 {
        // Read JSON from file
        match fs::read_to_string(&args[1]) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("File reading error: {}", e);
                return;
            }
        }
    } else {
        // Read JSON from standard input
        println!("Enter JSON (end with Ctrl+D):");
        let mut buffer = String::new();
        match io::stdin().read_to_string(&mut buffer) {
            Ok(_) => buffer,
            Err(e) => {
                eprintln!("Standard input reading error: {}", e);
                return;
            }
        }
    };

    match parse_json(&json_str) {
        Ok(json) => println!("Parsed JSON: {}", json),
        Err(e) => eprintln!("JSON parsing error: {}", e),
    }
}
