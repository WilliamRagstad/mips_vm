//! Debugger core functionality

use mips_vm::vm::VM;
use mips_vm::memory::MemorySegment;
use mips_vm::program::Program;
use std::path::Path;

pub struct Debugger {
    vm: Option<VM>,
    breakpoints: Vec<u32>,
    running: bool,
    current_pc: u32,
    registers: [u32; 32],
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            vm: None,
            breakpoints: Vec::new(),
            running: false,
            current_pc: 0,
            registers: [0; 32],
        }
    }

    pub fn load_program(&mut self, path: &Path) -> Result<(), DebuggerError> {
        // TODO: Load ELF file and create Program
        println!("Loading program: {}", path.display());
        
        // For now, create a dummy program
        let program = Program {
            data_section: mips_vm::program::DataSection {
                initialized: Vec::new(),
            },
            text_section: mips_vm::program::TextSection {
                blocks: Vec::new(),
                global_labels: Vec::new(),
            },
        };
        let mmio = Vec::new();
        self.vm = Some(VM::new(program, mmio));
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), DebuggerError> {
        self.running = true;
        
        // TODO: Actually run the VM
        println!("Running program...");
        
        Ok(())
    }

    pub fn start_interactive(&mut self) -> Result<(), DebuggerError> {
        crate::ui::start_tui(self)
            .map_err(|e| DebuggerError::UiError(e.to_string()))
    }

    pub fn step(&mut self) -> Result<(), DebuggerError> {
        // TODO: Implement single step execution
        self.current_pc += 4;
        Ok(())
    }

    pub fn add_breakpoint(&mut self, address: u32) {
        if !self.breakpoints.contains(&address) {
            self.breakpoints.push(address);
        }
    }

    pub fn remove_breakpoint(&mut self, address: u32) {
        self.breakpoints.retain(|&bp| bp != address);
    }

    pub fn get_registers(&self) -> &[u32; 32] {
        &self.registers
    }

    pub fn get_pc(&self) -> u32 {
        self.current_pc
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DebuggerError {
    #[error("UI error: {0}")]
    UiError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
