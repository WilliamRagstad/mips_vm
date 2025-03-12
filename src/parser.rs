use std::collections::HashMap;

use colorful::Colorful;
use pest::Parser;
use pest_derive::Parser;

use crate::program::{
    Block, DataSection, Instruction, InstructionArg, InstructionKind, Offset, Program, RawData,
    Register, Section, TextSection,
};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MainParser;

pub fn parse(input: &str) -> Option<Program> {
    match MainParser::parse(Rule::program, input) {
        Ok(pairs) => {
            assert_eq!(pairs.clone().count(), 1);
            let program = pairs.clone().next().unwrap();
            let pairs = program.into_inner();
            for pair in pairs.clone() {
                log::trace!("\n======= PAIR =======");
                log::trace!("Rule:    {:?}", pair.as_rule());
                log::trace!("Span:    {:?}", pair.as_span());
                log::trace!("Text:    '{}'", pair.as_str().trim().yellow());
                log::trace!("Inner:   {:?}", pair.into_inner().collect::<Vec<_>>());
            }
            let mut prog = Program {
                data: DataSection {
                    globals: HashMap::new(),
                },
                text: TextSection {
                    blocks: Vec::new(),
                    global_labels: Vec::new(),
                },
            };
            let mut offset: Offset = 0;
            let mut current_section: Option<Section> = None;
            let mut current_block: Option<Block> = None;
            for pair in pairs {
                match pair.as_rule() {
                    Rule::directive => {
                        let mut inner = pair.into_inner();
                        let inner_first = inner.next().unwrap();
                        log::trace!("Directive: {:?}", inner);
                        match inner_first.as_rule() {
                            Rule::section_directive => {
                                let section = match inner_first.as_str().trim() {
                                    ".data" => Section::Data,
                                    ".text" => Section::Text,
                                    _ => unreachable!(),
                                };
                                log::trace!("Section: {:?}", section);
                                current_section = Some(section);
                            }
                            Rule::text_directive => {
                                let directive = inner_first.as_str().trim();
                                let symbol = match directive {
                                    ".global" | ".globl" => {
                                        let value = inner.next().unwrap().as_str().to_string();
                                        value
                                    }
                                    _ => unreachable!(),
                                };
                                log::trace!(
                                    "  - Directive: {:?} with data: {:?}",
                                    directive,
                                    symbol
                                );
                                prog.text.global_labels.push(symbol);
                            }
                            _ => unreachable!(),
                        }
                    }
                    Rule::label => {
                        let source = pair.as_str().trim().to_string();
                        let mut inner = pair.into_inner();
                        let label = inner
                            .next()
                            .expect("Expected label identifier")
                            .as_str()
                            .to_string();
                        log::trace!("Label: {:?}", label);
                        if current_section == Some(Section::Data) {
                            if let Some(inner_directive) = inner.next() {
                                let directive = inner_directive.as_str().trim();
                                let data = match directive {
                                    ".asciiz" => {
                                        let data =
                                            inner.next().unwrap().as_str().to_string().into_bytes();
                                        // remove first and last character (")
                                        let mut data = data[1..data.len() - 1].to_vec();
                                        data.push(0); // null-terminated string
                                        log::trace!(".asciiz {:?}", &data);
                                        RawData {
                                            offset,
                                            source,
                                            data,
                                        }
                                    }
                                    ".ascii" => {
                                        let data =
                                            inner.next().unwrap().as_str().to_string().into_bytes();
                                        // remove first and last character (")
                                        let data = data[1..data.len() - 1].to_vec();
                                        log::trace!(".ascii {:?}", &data);
                                        RawData {
                                            offset,
                                            source,
                                            data,
                                        }
                                    }
                                    ".word" => {
                                        let word: i32 =
                                            inner.next().unwrap().as_str().parse().unwrap();
                                        log::trace!(".word {:?}", word);
                                        RawData {
                                            offset,
                                            source,
                                            data: word.to_le_bytes().to_vec(),
                                        }
                                    }
                                    ".byte" => {
                                        let byte: u8 =
                                            inner.next().unwrap().as_str().parse().unwrap();
                                        log::trace!(".byte {:?}", byte);
                                        RawData {
                                            offset,
                                            source,
                                            data: vec![byte],
                                        }
                                    }
                                    _ => unreachable!(),
                                };
                                offset += data.size() as Offset;
                                log::trace!("  - Directive: {:?} with data: {:?}", directive, data);
                                prog.data.globals.insert(label, data);
                            } else {
                                unreachable!();
                            }
                        } else if current_section == Some(Section::Text) {
                            if let Some(previous_block) = current_block.take() {
                                log::trace!("Pushing block: {:?}", previous_block);
                                prog.text.blocks.push(previous_block);
                            }
                            current_block = Some(Block {
                                offset,
                                label,
                                instructions: Vec::new(),
                            });
                        } else {
                            unreachable!();
                        }
                    }
                    Rule::instruction => {
                        log::trace!("Instruction: {:?}", pair);
                        let mut inner = pair.into_inner();
                        let kind = InstructionKind::from(inner.next().unwrap().as_str());
                        let mut args: Vec<InstructionArg> = Vec::new();
                        for arg in inner {
                            log::trace!("  - Arg: {:?}", arg);
                            match arg.as_rule() {
                                Rule::register => args
                                    .push(InstructionArg::Register(Register::from(arg.as_str()))),
                                Rule::immediate => args
                                    .push(InstructionArg::Immediate(arg.as_str().parse().unwrap())),
                                Rule::identifier => {
                                    args.push(InstructionArg::Label(arg.as_str().to_string()))
                                }
                                _ => unreachable!(),
                            }
                        }
                        log::trace!("  - Kind: {:?}", kind);
                        log::trace!("  - Args: {:?}", args);
                        let Some(block) = current_block.as_mut() else {
                            unreachable!();
                        };
                        let instr = Instruction { offset, kind, args };
                        offset += instr.size() as Offset;
                        block.instructions.push(instr);
                    }
                    Rule::EOI => {}
                    _ => unreachable!(),
                }
            }
            if let Some(previous_block) = current_block.take() {
                log::trace!("Pushing final block: {:?}", previous_block);
                prog.text.blocks.push(previous_block);
            }
            Some(prog)
        }
        Err(e) => {
            log::trace!("{}", e);
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parse;

    #[test]
    fn test() {
        let input = include_str!("../tests/prog1.asm");
        let prog = parse(input);
        assert_ne!(prog, None);
        let prog = prog.unwrap();
        println!("{}", prog.show());
    }
}
