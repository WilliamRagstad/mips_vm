use interpreter::execute;
use parser::parse;

pub mod interpreter;
pub mod memory;
pub mod parser;
pub mod program;
pub mod registers;

fn main() {
    env_logger::init();
    let input = std::env::args().nth(1).expect("No input file provided");
    let input = std::fs::read_to_string(input).expect("Failed to read input file");
    if let Some(program) = parse(&input) {
        let entry = program
            .text
            .entry_block()
            .expect("No entry block found")
            .address;
        execute(program, entry);
    }
}
