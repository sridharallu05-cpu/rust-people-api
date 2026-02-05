use std::fs;

fn main() {
    let content = fs::read_to_string("names.txt")
        .expect("Could not read file");

    for name in content.lines() {
        println!("Hello {}", name);
    }
}
