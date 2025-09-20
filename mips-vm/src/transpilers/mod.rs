use std::fs::File;

use crate::{compiler::Target, program::Program};

pub mod nasm;

pub trait Transpiler {
    /// Transpile the MIPS program to assembly code.
    /// Same process for both ELF and PE targets.
    fn transpile(
        &self,
        program: &Program,
        target: &Target,
        output: &mut File,
    ) -> Result<String, String>;
}
