//! MIPS TUI Debugger
//! 
//! A terminal-based debugger for MIPS programs with ncurses-like interface

mod debugger;
mod ui;
mod commands;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mips-dbg")]
#[command(about = "MIPS TUI Debugger")]
struct Args {
    /// ELF file to debug
    #[arg(value_name = "FILE")]
    file: PathBuf,
    
    /// Start debugging immediately
    #[arg(short, long)]
    run: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    let mut debugger = debugger::Debugger::new();
    debugger.load_program(&args.file)?;
    
    if args.run {
        debugger.run()?;
    } else {
        debugger.start_interactive()?;
    }
    
    Ok(())
}
