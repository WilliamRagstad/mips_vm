use clap::{Parser, Subcommand};
use mips_vm::{memory::PAGE_SIZE, parser::parse, vm::VM};

mod mmio;

/// Simple program to run MIPS VM
#[derive(Parser)]
#[command(name = "mips", bin_name = "mips")]
#[command(about = "A simple MIPS VM", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile the input file
    #[command(name = "compile", alias = "c")]
    Compile {
        /// Input file to compile
        input: String,
        /// Output file for the compiled program
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Run the input file
    #[command(name = "run", alias = "r")]
    Run {
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
    },
}

fn main() {
    log_init();
    let args = Cli::parse();

    match args.command {
        Commands::Compile { input, output } => {
            let input_content = std::fs::read_to_string(&input).expect("Failed to read input file");
            if let Some(program) = parse(&input_content) {
                let mmio = Vec::new();
                let vm = VM::new(program, mmio);
                let output = if let Some(output) = output {
                    std::path::PathBuf::from(output)
                } else {
                    let mut path = std::path::PathBuf::from(input);
                    path.set_extension("bin");
                    path
                };
                let dump = vm.memory().dump(true, PAGE_SIZE >> 5, true);
                std::fs::write(&output, dump).unwrap();
                println!(
                    "Compilation successful! Output written to {}",
                    output.display()
                );
            } else {
                eprintln!("Failed to compile the input file.");
            }
        }
        Commands::Run {
            input,
            dump_file,
            non_compressed,
            shard_size,
        } => {
            let input_content = std::fs::read_to_string(input).expect("Failed to read input file");
            if let Some(program) = parse(&input_content) {
                let mmio = Vec::new();
                let mut vm = VM::new(program, mmio);

                if let Some(dump_file) = dump_file {
                    let dump = vm.memory().dump(!non_compressed, shard_size, false);
                    let dump_path = std::path::PathBuf::from(dump_file);
                    std::fs::write(&dump_path, dump).unwrap();
                }

                vm.execute(vm.entrypoint().expect("No entrypoint found"));
            }
        }
    }
}

fn log_init() {
    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .init();
}
