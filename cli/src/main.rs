use clap::Parser;
use mips_vm::{parser::parse, vm::VM};

/// Simple program to run MIPS VM
#[derive(Parser)]
#[command(name = "mips", bin_name = "mips")]
#[command(about = "A simple MIPS VM", long_about = None)]
struct Cli {
    /// Input file to run
    input: String,
}

fn main() {
    log_init();
    let args = Cli::parse();
    let input = std::fs::read_to_string(args.input).expect("Failed to read input file");
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
