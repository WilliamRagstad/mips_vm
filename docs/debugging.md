# CLI debugging walkthrough

From the repository root, with Rust stable installed:

```sh
cargo test --workspace
cargo run -p mips_cli -- debug examples/debug.asm
```

Enter:

```text
break loop
continue
step
memory value 4
registers
continue
delete loop
continue
quit
```

The first continue stops **before** the store at `loop` (PC `0x00400008`).
Step executes that store and advances to `0x0040000C`. Memory shows
`0x10010000: 02 00 00 00`, and `$t0` is still 2. Continue stops at the next
iteration with `$t0` equal to 1. Delete the breakpoint and continue: the program
halts at `0x0040001C`, with `$t0` equal to 0 and `value` equal to 1.
This exact session is covered by `cli/tests/cli.rs`.

Commands: `step/s`, `continue/c`, `break/b LABEL|ADDRESS`,
`delete/d LABEL|ADDRESS`, `breaks`, `registers/r`,
`memory/m LABEL|ADDRESS COUNT`, `help/h`, `quit/q`.
Addresses/counts accept decimal or `0x` hexadecimal. Breakpoints require an
instruction address; memory counts must be 1–256. Bad commands/accesses report
errors and leave the prompt usable. EOF exits. No restart or mutation commands.

Continue stops before breakpoints. Continuing from a breakpoint executes that
instruction once before checking again, so loops can hit the same breakpoint
repeatedly. Step ignores breakpoints. Continue pauses after 100,000 instructions
to keep infinite loops manageable; use continue again to resume. Blocking input
syscalls and MMIO still wait for host input. Debugger and guest share stdin;
interactive guest input must be supplied when the guest requests it.

Normal execution remains available:

```sh
cargo run -p mips_cli -- run examples/hello_world.asm
cargo run -p mips_cli -- run examples/mem_text.asm
printf X | cargo run -p mips_cli -- run examples/mmio_hello_world.asm
```

Expected outputs: `Hello, world!` plus newline; `76`; and the MMIO prompt followed
by `X`. The MMIO example now exits on EOF/NUL using a blocking CLI adapter.
The memory example uses a mapped data word instead of its previous unmapped
address/oversized literal. See [dialect and reference comparison](dialect.md)
for compatibility limits.
