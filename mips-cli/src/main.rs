use clap::{Parser, Subcommand};
use mips_vm::{compiler::Compiler, parser::parse, vm::VM};
use mips_asm::{parse_assembly, Assembler, AssemblerConfig};
use mips_elf::ElfType;

mod mmio;

/// Simple program to run MIPS VM
#[derive(Parser)]
#[command(name = "mips", bin_name = "mips")]
#[command(about = "A simple MIPS VM", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::ValueEnum, Clone, Debug)]
#[clap(rename_all = "lower")]
enum Target {
    #[allow(clippy::upper_case_acronyms)]
    ELF,
    PE,
}

#[derive(clap::ValueEnum, Clone, Debug)]
#[clap(rename_all = "lower")]
enum OutputType {
    Executable,
    Relocatable,
}

impl From<Target> for mips_vm::compiler::Target {
    fn from(target: Target) -> Self {
        match target {
            Target::ELF => mips_vm::compiler::Target::ELF,
            Target::PE => mips_vm::compiler::Target::PE,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Assemble the input file using new assembler
    #[command(name = "assemble", alias = "a")]
    Assemble {
        /// Input assembly file
        input: String,
        /// Output file for the assembled program
        #[arg(short, long)]
        output: Option<String>,
        /// Output format (executable or relocatable)
        #[arg(short = 't', long, value_enum)]
        output_type: Option<OutputType>,
    },
    /// Compile the input file
    #[command(name = "compile", alias = "c")]
    Compile {
        /// Input file to compile
        input: String,
        /// Output file for the compiled program
        #[arg(short, long)]
        output: Option<String>,
        /// Target file format.
        #[arg(short, long, value_enum)]
        target: Target,
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
        Commands::Assemble {
            input,
            output,
            output_type,
        } => {
            let input_content = std::fs::read_to_string(&input).expect("Failed to read input file");
            let parsed_program = parse_assembly(&input_content).expect("Failed to parse assembly");
            
            let output_type = output_type.unwrap_or(OutputType::Executable);
            let elf_type = match output_type {
                OutputType::Executable => ElfType::Executable,
                OutputType::Relocatable => ElfType::Relocatable,
            };
            
            let config = AssemblerConfig {
                output_type: elf_type,
                debug_info: false,
                optimize: false,
            };
            
            let mut assembler = Assembler::new(config);
            let assembled = assembler.assemble(&parsed_program).expect("Failed to assemble");
            
            let output = if let Some(output) = output {
                std::path::PathBuf::from(output)
            } else {
                let mut path = std::path::PathBuf::from(input);
                path.set_extension("bin");
                path
            };
            
            std::fs::write(&output, &assembled.code).expect("Failed to write output file");
            println!(
                "Assembly successful! Output written to {} ({} bytes)",
                output.display(),
                assembled.code.len()
            );
        }
        Commands::Compile {
            input,
            output,
            target,
        } => {
            let input_content = std::fs::read_to_string(&input).expect("Failed to read input file");
            if let Some(program) = parse(&input_content) {
                let output = if let Some(output) = output {
                    std::path::PathBuf::from(output)
                } else {
                    let mut path = std::path::PathBuf::from(input);
                    path.set_extension("bin");
                    path
                };
                let compiler = Compiler::new(program);
                compiler
                    .compile(target.into(), &output)
                    .expect("Failed to compile");
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
