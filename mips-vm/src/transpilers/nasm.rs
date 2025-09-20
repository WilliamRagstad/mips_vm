use crate::program::Program;

use super::Transpiler;

#[derive(Debug, Clone, Default)]
pub struct NasmTranspiler {}

impl Transpiler for NasmTranspiler {
    fn transpile(
        &self,
        program: &Program,
        target: &crate::compiler::Target,
        output: &mut std::fs::File,
    ) -> Result<String, String> {
        todo!()
    }
}
