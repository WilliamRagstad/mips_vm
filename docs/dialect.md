# Supported interpreter dialect

This is a small educational MIPS32-style **assembly IR interpreter**, not a
complete MIPS ISA implementation or a binary emulator. Architecture and JIT work
should wait for conformance coverage. The compiler/encoded instruction dump is
experimental and is not the execution oracle; pseudo-ops occupy one IR word.

## Execution contract

- 32-bit registers, little-endian memory, immutable reads of `$zero`. Registers
  initially read zero (including `$sp`; programs must initialize it before use).
- PC starts at the exact label `main`, then `entry`, then `start`, otherwise the
  beginning of text (`0x00400000`). Each IR instruction occupies four bytes.
- **No delay slots.** Branches to labels are absolute IR targets. `jal`/`jalr`
  link to PC+4, and `jr` actually transfers control. A branch does not execute
  the following instruction when taken. All control targets must be aligned
  executable instructions.
- Falling off the text end and syscalls 10/17 halt. Stepping a halted VM is
  idempotent. Faults return an error with PC on the faulting instruction.
  Syscall output/input already performed before a fault cannot be rolled back.
- `add`, `addi`, `sub` trap signed overflow; unsigned variants wrap. `slt` and
  `slti` compare signed values; `sltu` and `sltiu` compare unsigned values.
  `sra` sign extends, `srl` zero fills. Shift counts use the low five bits.
- `mul d,s,t` returns the low 32 bits in `d`; there are no HI/LO registers.
- `lb` sign extends, `lbu` zero extends, `lw`/`sw` require word alignment.
  Addresses use `offset(base)`; offsets are signed 16-bit bit patterns.
  Unmapped, out-of-segment and protected accesses return errors.
- Static data begins at `0x10010000`. `la` resolves a label's address. Syscall 9
  allocates readable/writable heap pages. Memory inspection is read-only,
  requires 1–256 bytes within one mapped segment, and refuses device reads.

## Source syntax supported by the CLI

Lowercase mnemonics and named registers (`$t0`, `$ra`, etc.; `$0` aliases `$zero`):

```
add addi addiu addu sub subu mul
and andi or ori xor xori nor
slt slti sltiu sltu sll sllv srl srlv sra
beq bne j jal jr jalr
lb lbu lw sb sw lui
li la move nop syscall
```

Arithmetic uses `destination, source, source/immediate`; `jalr` uses
`destination, source`. `beq`/`bne` use two registers and a **label**.
Loads/stores use `register, offset(base)`. Immediates are unsigned 16-bit source
literals (decimal, `0x`, `0b`). Use `65535` for -1 in `addi`/`addiu`, signed
comparisons, or offsets; construct larger constants with `lui`/`ori`.
`li` zero extends its 16-bit literal. Logical immediates zero extend.
The library has additional IR instruction variants which are not promised CLI
syntax. No floating point, coprocessors, exceptions/interrupts, or HI/LO support.

Use `.data`, `.text`, `.globl`/`.global`, labels, `#` comments, and a final newline.
Data examples use single-value `.word`/`.byte`, `.ascii`, `.asciiz`, and `.space`.
The existing parser/assembler still assumes well-formed operand shapes and can
panic on malformed or out-of-range source; runtime fault handling does not yet
make the assembler a robust untrusted-input interface.

## MARS reference comparison

Compared against the published [MARS 4.5 introduction](https://dpetersanderson.github.io/Help/MarsHelpIntro.html),
[settings](https://dpetersanderson.github.io/Help/MarsHelpSettings.html), and
[syscall table](https://dpetersanderson.github.io/Help/SyscallHelp.html).
These are specification comparisons; automated tests run this VM, not MARS.

| Area | This VM | MARS reference |
| --- | --- | --- |
| Delay slots | Always disabled; link PC+4 | Disabled by default; can enable delayed branching, executing the instruction after a transfer |
| Pseudo-ops | One four-byte IR instruction | May expand into several machine instructions; absolute PCs need not match |
| Syscalls 1, 4, 5, 10, 11, 12 | Signed integer and ASCII string/character console I/O and exit | Same service numbers and integer register conventions |
| Syscall 8 | Reads a host line, truncates to n-1 bytes, appends NUL; n=0 writes nothing | fgets-style input; long-line handling may retain input differently |
| Syscall 9 | Byte-sized heap allocation, no automatic alignment | MARS sbrk has its own allocation/alignment behavior |
| Syscall 17 | Halts; ignores `$a0` as host exit status | Exit status used by command-line MARS; ignored by GUI |
| Syscalls 2, 3, 6, 7 | Explicit unsupported-syscall fault | Floating-point registers `$f12`/`$f0`; the old VM's integer-register emulation was incompatible |
| Other services | Unsupported-syscall fault | File I/O and MARS-specific services available |
| MMIO | CLI blocking console adapter; ready flags always 1; EOF/NUL is zero | Device UI, readiness and timing model differ |

For a manual comparison, open `examples/debug.asm` in MARS with delayed branching
**off**, step through the loop, and compare `$t0` (2 → 1 → 0) and `value`
(0 → 2 → 1). Compare labels/values rather than absolute PCs because `la` expands.
Enabling delay slots makes the instruction after `bne` execute before the branch.
`examples/hello_world.asm` should print the same ASCII text using services 4/10.

## Conformance evidence

`cargo test --workspace` checks register/memory/PC transitions, signed arithmetic
and shifts, wrapping, forward/backward taken/untaken branches, calls/returns,
invalid access, overflow, unsupported syscalls, halt idempotence, page-boundary
bytes, breakpoints/resume, command validation, all examples, and the walkthrough.
This is a baseline suite, not an exhaustive proof of ISA compliance.
