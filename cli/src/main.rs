use mips_vm::{parser::parse, vm::VM};

fn main() {
    log_init();
    let input = std::env::args().nth(1).expect("No input file provided");
    let input = std::fs::read_to_string(input).expect("Failed to read input file");
    if let Some(program) = parse(&input) {
        let mut vm = VM::new(program);
        vm.execute(vm.entrypoint().expect("No entrypoint found"));
    }
}

fn log_init() {
    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .init();
}
