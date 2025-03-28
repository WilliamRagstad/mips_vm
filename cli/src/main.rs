use clap::Parser;
use mips_vm::{parser::parse, vm::VM};

mod mmio;

/// Simple program to run MIPS VM
#[derive(Parser)]
#[command(name = "mips", bin_name = "mips")]
#[command(about = "A simple MIPS VM", long_about = None)]
struct Cli {
    /// Input file to run
    input: String,
    /// Optional memory dump to file
    #[arg(short, long)]
    dump_file: Option<String>,
    /// Do not compress memory dump
    #[arg(short, long, default_value = "false")]
    non_compressed: bool,
    /// Shard size for compression
    #[arg(short, long, default_value = "128")]
    shard_size: usize,
}

fn main() {
    log_init();
    let args = Cli::parse();
    let input = std::fs::read_to_string(args.input).expect("Failed to read input file");
    if let Some(program) = parse(&input) {
        let mmio = Vec::new();
        let mut vm = VM::new(program, mmio);

        if let Some(dump_file) = args.dump_file {
            let dump = vm.memory().dump(!args.non_compressed, args.shard_size);
            let dump_path = std::path::PathBuf::from(dump_file);
            std::fs::write(&dump_path, dump).unwrap();
        }

        vm.execute(vm.entrypoint().expect("No entrypoint found"));
    }
}

fn log_init() {
    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .init();
}
