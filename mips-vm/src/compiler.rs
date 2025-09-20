use std::{
    io::Write,
    path::{Display, Path},
    process::Command,
};

use crate::{
    program::Program,
    transpilers::{nasm::NasmTranspiler, Transpiler},
};

pub enum Target {
    #[allow(clippy::upper_case_acronyms)]
    ELF,
    PE,
}

impl Target {
    pub fn target_triplet(&self) -> &str {
        match self {
            Target::ELF => "x86_64-unknown-linux-gnu",
            Target::PE => "x86_64-pc-windows-msvc",
        }
    }
}

pub struct Compiler {
    program: Program,
    transpiler: NasmTranspiler,
}

impl Compiler {
    pub fn new(program: Program) -> Self {
        Compiler {
            program,
            transpiler: NasmTranspiler::default(),
        }
    }

    pub fn compile(&self, target: Target, output: &Path) -> Result<(), String> {
        let mut assembly_output = output.to_path_buf();
        if assembly_output.extension().is_none() {
            match target {
                Target::ELF => assembly_output.set_extension("elf.asm"),
                Target::PE => assembly_output.set_extension("pe.asm"),
            };
        }

        let mut file = std::fs::File::create(&assembly_output).map_err(|e| e.to_string())?;
        let assembly_text = self
            .transpiler
            .transpile(&self.program, &target, &mut file)
            .map_err(|e| format!("Failed to transpile: {}", e))?;
        file.write_all(assembly_text.as_bytes())
            .map_err(|e| format!("Failed to write to file: {}", e))?;
        file.flush()
            .map_err(|e| format!("Failed to flush file: {}", e))?;
        file.sync_all()
            .map_err(|e| format!("Failed to sync file: {}", e))?;
        println!("Assembly code written to: {}", assembly_output.display());
        // Now we can assemble the code using nasm or any other assembler
        // For example, using nasm:
        let status = Command::new("nasm")
            .arg("-f")
            .arg(target.target_triplet())
            .arg("-o")
            .arg(&assembly_output)
            .status()
            .map_err(|e| format!("Failed to run nasm: {}", e))?;
        if !status.success() {
            return Err(format!("Nasm failed with status: {}", status));
        }
        // Now we can link the object file to create the final executable
        let output_file = output.to_path_buf();
        let status = Command::new("ld")
            .arg("-o")
            .arg(&output_file)
            .arg(assembly_output.with_extension("o"))
            .status()
            .map_err(|e| format!("Failed to run ld: {}", e))?;
        if !status.success() {
            return Err(format!("Linker failed with status: {}", status));
        }
        // Clean up the intermediate files
        std::fs::remove_file(assembly_output.with_extension("o"))
            .map_err(|e| format!("Failed to remove intermediate file: {}", e))?;
        std::fs::remove_file(assembly_output)
            .map_err(|e| format!("Failed to remove intermediate file: {}", e))?;
        println!("Executable created at: {}", output_file.display());
        Ok(())
    }
}
