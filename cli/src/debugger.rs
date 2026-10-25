use mips_vm::{
    address::Address,
    registers::Register,
    vm::{Step, VM},
};
use std::{
    collections::BTreeSet,
    io::{self, BufRead, Write},
};

const HELP: &str = "step (s), continue (c), break (b) LABEL|ADDRESS, delete (d) LABEL|ADDRESS, breaks, registers (r), memory (m) ADDRESS COUNT (1..256), quit (q), help (h)";
fn number(s: &str) -> Result<u32, String> {
    if let Some(hex) = s.strip_prefix("0x") {
        u32::from_str_radix(hex, 16)
    } else {
        s.parse()
    }
    .map_err(|_| format!("Invalid number: {s}"))
}
fn address(vm: &VM, s: &str) -> Result<Address, String> {
    vm.memory()
        .address_of_label(s)
        .or_else(|_| {
            number(s)
                .map(Address::new)
                .map_err(|_| mips_vm::memory::MemoryError::InvalidAddress)
        })
        .map_err(|_| format!("Unknown label or address: {s}"))
}
fn location(vm: &VM, out: &mut impl Write) -> io::Result<()> {
    let instruction = vm
        .memory()
        .execute(vm.pc())
        .map(|i| i.show())
        .unwrap_or_default();
    writeln!(
        out,
        "PC {} {}{}",
        vm.pc(),
        if vm.halted() { "halted " } else { "" },
        instruction
    )
}
pub fn run(vm: &mut VM, input: &mut impl BufRead, out: &mut impl Write) -> io::Result<()> {
    let mut breaks = BTreeSet::new();
    let mut stopped_at_break = None;
    writeln!(out, "{HELP}")?;
    location(vm, out)?;
    loop {
        write!(out, "debug> ")?;
        out.flush()?;
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            return Ok(());
        }
        let parts = line.split_whitespace().collect::<Vec<_>>();
        let result: Result<(), String> = (|| {
            match parts.as_slice() {
                [] => {}
                ["q" | "quit"] => return Ok(()),
                ["h" | "help"] => writeln!(out, "{HELP}").map_err(|e| e.to_string())?,
                ["b" | "break", target] => {
                    let a = address(vm, target)?;
                    vm.memory()
                        .execute(a)
                        .map_err(|e| format!("Invalid breakpoint: {e:?}"))?;
                    breaks.insert(a);
                    writeln!(out, "Breakpoint at {a}").map_err(|e| e.to_string())?;
                }
                ["d" | "delete", target] => {
                    breaks.remove(&address(vm, target)?);
                }
                ["breaks"] => {
                    for a in &breaks {
                        writeln!(out, "{a}").map_err(|e| e.to_string())?;
                    }
                }
                ["r" | "registers"] => {
                    for name in [
                        "$zero", "$at", "$v0", "$v1", "$a0", "$a1", "$a2", "$a3", "$t0", "$t1",
                        "$t2", "$t3", "$t4", "$t5", "$t6", "$t7", "$s0", "$s1", "$s2", "$s3",
                        "$s4", "$s5", "$s6", "$s7", "$t8", "$t9", "$k0", "$k1", "$gp", "$sp",
                        "$fp", "$ra",
                    ] {
                        let v = vm.registers().get(&Register::from(name));
                        writeln!(out, "{name:5} 0x{v:08x} ({})", v as i32)
                            .map_err(|e| e.to_string())?;
                    }
                    location(vm, out).map_err(|e| e.to_string())?;
                }
                ["m" | "memory", addr, count] => {
                    let a = address(vm, addr)?;
                    let bytes = vm
                        .inspect(a, number(count)? as usize)
                        .map_err(|e| format!("{e:?}"))?;
                    writeln!(
                        out,
                        "{a}: {}",
                        bytes
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                    .map_err(|e| e.to_string())?;
                }
                [command @ ("s" | "step" | "c" | "continue")] => {
                    let continuing = matches!(*command, "c" | "continue");
                    let skip = stopped_at_break.take();
                    for n in 0..100_000 {
                        if continuing
                            && !vm.halted()
                            && breaks.contains(&vm.pc())
                            && !(n == 0 && skip == Some(vm.pc()))
                        {
                            stopped_at_break = Some(vm.pc());
                            writeln!(out, "Breakpoint at {}", vm.pc())
                                .map_err(|e| e.to_string())?;
                            break;
                        }
                        let state = vm
                            .step()
                            .map_err(|e| format!("Fault at {}: {e:?}", vm.pc()))?;
                        if state == Step::Halted || !continuing {
                            break;
                        }
                        if n == 99_999 {
                            writeln!(out, "Paused after 100000 instructions; continue to resume.")
                                .map_err(|e| e.to_string())?;
                        }
                    }
                    location(vm, out).map_err(|e| e.to_string())?;
                }
                _ => return Err(format!("Invalid command. {HELP}")),
            }
            Ok(())
        })();
        if matches!(parts.as_slice(), ["q" | "quit"]) {
            return Ok(());
        }
        if let Err(e) = result {
            writeln!(out, "{e}")?;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn break_resume_step_inspect_and_terminate() {
        let mut vm = VM::new(
            mips_vm::parser::parse(
                ".text\nmain:\nli $t0, 2\nloop:\naddi $t0, $t0, 65535\nbne $t0, $zero, loop\n",
            )
            .unwrap(),
            vec![],
        );
        let mut output = vec![];
        run(
            &mut vm,
            &mut &b"b main\nb loop\nc\nc\nr\nc\ns\nm 0 4\nm 0x400000 257\nd loop\nc\ns\nq\n"[..],
            &mut output,
        )
        .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("$t0   0x00000002"));
        assert!(output.matches("Breakpoint at 0x00400004").count() >= 3);
        assert!(output.contains("InvalidSize"));
        assert!(vm.halted());
        assert_eq!(vm.registers().get(&Register::T0), 0);
    }
    #[test]
    fn eof_and_bad_commands_do_not_execute() {
        let mut vm = VM::new(
            mips_vm::parser::parse(".text\nmain:\nnop\n").unwrap(),
            vec![],
        );
        run(&mut vm, &mut &b"b 1\ns extra\nm 0 -1\n"[..], &mut vec![]).unwrap();
        assert_eq!(vm.pc().unwrap(), 0x400000);
    }
}
