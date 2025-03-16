use parser::parse;
use vm::VM;

pub mod memory;
pub mod parser;
pub mod program;
pub mod registers;
pub mod vm;

fn main() {
    log_init();
    let input = std::env::args().nth(1).expect("No input file provided");
    let input = std::fs::read_to_string(input).expect("Failed to read input file");
    if let Some(program) = parse(&input) {
        let entrypoint = program
            .text
            .entry_block()
            .expect("No entry block found")
            .address;
        let mut vm = VM::new(program);
        vm.execute(entrypoint);
    }
}

fn log_init() {
    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .init();
}
