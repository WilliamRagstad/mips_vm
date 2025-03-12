use interpreter::execute;
use parser::parse;

pub mod interpreter;
pub mod parser;
pub mod program;

fn main() {
    let input = std::env::args().nth(1).expect("No input file provided");
    let input = std::fs::read_to_string(input).expect("Failed to read input file");
    if let Some(program) = parse(&input) {
        log::debug!("Executing {}", program.show());
        execute(program);
    }
}
