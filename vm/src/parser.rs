use colorful::Colorful;
use pest::Parser;
use pest_derive::Parser;

use crate::{
    program::{
        Block, DataSection, Immediate, Instruction, InstructionArg, InstructionKind, Program,
        Section, StaticData, TextSection,
    },
    registers::Register,
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
            log::trace!("{}", "======= PAIRS =======".blue());
            for pair in pairs.clone() {
                log::trace!("{}", "------- PAIR -------".cyan());
                log::trace!("Rule:    {:?}", pair.as_rule());
                log::trace!("Span:    {:?}", pair.as_span());
                log::trace!("Text:    '{}'", pair.as_str().trim().yellow());
                log::trace!("Inner:   {:?}", pair.into_inner().collect::<Vec<_>>());
            }
            log::trace!("{}", "======= PROGRAM =======".blue());
            let mut prog = Program {
                data_section: DataSection {
                    initialized: Vec::new(),
                },
                text_section: TextSection {
                    blocks: Vec::new(),
                    global_labels: Vec::new(),
                },
            };
            let mut current_section: Option<Section> = None;

            // Unnamed initial block
            let mut current_block: Block = Block {
                label: String::new(),
                instructions: Vec::new(),
            };

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
                                prog.text_section.global_labels.push(symbol);
                            }
                            _ => unreachable!(),
                        }
                    }
                    Rule::label => {
                        let mut inner = pair.into_inner();
                        let label = inner
                            .next()
                            .expect("Expected label identifier")
                            .as_str()
                            .to_string();
                        let source = inner.as_str().trim().to_string();
                        log::trace!("Label: {:?}, source: {}", label, source.clone().yellow());
                        if current_section == Some(Section::Data) {
                            if let Some(inner_directive) = inner.next() {
                                let directive = inner_directive.as_str().trim();
                                let data = match directive {
                                    ".asciiz" => {
                                        let str = unescape_string(inner.next().unwrap().as_str());
                                        let data = str.into_bytes();
                                        // remove first and last character (")
                                        let mut data = data[1..data.len() - 1].to_vec();
                                        data.push(0); // null-terminated string
                                        log::trace!(".asciiz {:?}", &data);
                                        StaticData {
                                            label,
                                            source,
                                            data,
                                        }
                                    }
                                    ".ascii" => {
                                        let str = unescape_string(inner.next().unwrap().as_str());
                                        let data = str.into_bytes();
                                        // remove first and last character (")
                                        let data = data[1..data.len() - 1].to_vec();
                                        log::trace!(".ascii {:?}", &data);
                                        StaticData {
                                            label,
                                            source,
                                            data,
                                        }
                                    }
                                    ".word" => {
                                        let word: i32 =
                                            inner.next().unwrap().as_str().parse().unwrap();
                                        log::trace!(".word {:?}", word);
                                        StaticData {
                                            label,
                                            source,
                                            data: word.to_le_bytes().to_vec(),
                                        }
                                    }
                                    ".byte" => {
                                        let byte: u8 =
                                            inner.next().unwrap().as_str().parse().unwrap();
                                        log::trace!(".byte {:?}", byte);
                                        StaticData {
                                            label,
                                            source,
                                            data: vec![byte],
                                        }
                                    }
                                    _ => unreachable!(),
                                };
                                prog.data_section.initialized.push(data);
                            } else {
                                unreachable!();
                            }
                        } else if current_section == Some(Section::Text) {
                            log::trace!("Pushing block: {:?}", current_block);
                            prog.text_section.blocks.push(current_block);
                            current_block = Block {
                                label,
                                instructions: Vec::new(),
                            };
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
                                Rule::offset => {
                                    let mut inner = arg.into_inner();
                                    let immediate = inner.next().unwrap();
                                    let register = Register::from(inner.next().unwrap().as_str());
                                    args.push(InstructionArg::RegisterOffset(
                                        register,
                                        parse_imm(immediate),
                                    ));
                                }
                                Rule::immediate => {
                                    args.push(InstructionArg::Immediate(parse_imm(arg)))
                                }
                                Rule::identifier => {
                                    args.push(InstructionArg::Label(arg.as_str().to_string()))
                                }
                                _ => unreachable!(),
                            }
                        }
                        log::trace!("  - Kind: {:?}", kind);
                        log::trace!("  - Args: {:?}", args);

                        let instr = Instruction { kind, args };
                        current_block.instructions.push(instr);
                    }
                    Rule::EOI => {}
                    _ => unreachable!(),
                }
            }
            log::trace!("Pushing final block: {:?}", current_block);
            prog.text_section.blocks.push(current_block);

            Some(prog)
        }
        Err(e) => {
            println!("{}", e.to_string().light_red());
            None
        }
    }
}

/// immediate  = @{ hex | binary | integer }
// integer    = @{ (ASCII_DIGIT)+ }
// hex        = @{ "0x" ~ (ASCII_HEX_DIGIT)+ }
// binary     = @{ "0b" ~ ("0" | "1")+ }``
fn parse_imm(arg: pest::iterators::Pair<Rule>) -> Immediate {
    let arg = arg.as_str();
    if let Some(hex) = arg.strip_prefix("0x") {
        Immediate::from_str_radix(hex, 16).unwrap()
    } else if let Some(bin) = arg.strip_prefix("0b") {
        Immediate::from_str_radix(bin, 2).unwrap()
    } else {
        arg.parse().unwrap()
    }
}

fn unescape_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('0') => result.push('\0'),
                Some(c) => result.push(c),
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod test_parser {
    use super::parse;

    #[test]
    fn hello_world() {
        let input = include_str!("../../examples/hello_world.asm");
        let prog = parse(input);
        assert_ne!(prog, None);
        let prog = prog.unwrap();
        println!("{}", prog.show());
    }
}
