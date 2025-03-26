# MIPS VM

A *simple and experimental* MIPS virtual machine, currently interpreting assembly.

> MIPS (Microprocessor without Interlocked Pipelined Stages) is a family of reduced instruction set computer (RISC) instruction set architectures (ISA).<sup>[3](#references)</sup>

## Roadmap

<details>
<summary>Tasks</summary>

- [X] MIPS32 (I)
- [ ] MIPS64 (II)
- [X] Assembler
  - [X] Lexer
  - [X] Parser
  - [X] IR
- [X] Basic instruction set
  - [X] Arithmetic (`add`, `sub`, `mul`, `div`)
  - [X] Logical (`and`, `or`, `xor`, `nor`)
  - [X] Memory (`lw`, `sw`, `la`)
  - [X] Control flow (`beq`, `bne`, `j`, `jr`)
  - [X] System calls (`print_int`, `print_str`, `read_int`, `read_str`, `exit`)
- [X] Virtual machine (interpreter)
  - [X] Memory
  - [X] Registers
  - [X] Execution (Fetch-Decode-Execute)
  - [X] System calls
- [ ] Device drivers
  - [X] Memory mapped I/O (MMIO)
  - [ ] Interrupts
- [ ] Exception handling
- [ ] JIT compilation
- [ ] Compiler
  - [ ] Linker
    - [ ] Static linking
    - [ ] Dynamic linking
  - [ ] Register allocation
  - [ ] Instruction selection
  - [ ] Tail call optimization
  - [ ] Calling convention
  - [ ] Target architectures
    - [ ] RISC-V
    - [ ] ARM
    - [ ] x86
    - [ ] x86-64
    - [ ] WebAssembly
  - [ ] Target platforms
    - [ ] Windows (PE)
    - [ ] Linux (ELF)
- [ ] Debugger
- [ ] Profiler
- [ ] Multi-threading

On going tasks include performance optimizations, documentation, tests, examples and more.

</details>

## Example Usage

```nasm
.data
msg: .asciiz "Hello, world!"

.text
.globl main
main:
    li $v0, 4 # syscall 4 (print_str)
    la $a0, msg
    syscall

    li $v0, 10 # exit
    syscall
```

```bash
$ cargo run -- ./tests/prog1.asm
Hello, world!
```

## References

1. [MIPS IV Instruction Set - Charles Price](https://www.cs.cmu.edu/afs/cs/academic/class/15740-f97/public/doc/mips-isa.pdf)
2. [MIPS32 Architecture For Programmers Volume II-A: The MIPS32 Instruction Set](https://s3-eu-west-1.amazonaws.com/downloads-mips/documents/MD00086-2B-MIPS32BIS-AFP-06.03.pdf)
3. [MIPS Architecture - Wikipedia](https://en.wikipedia.org/wiki/MIPS_architecture)
4. [MIPS Assembly Language - Wikibooks](https://en.wikibooks.org/wiki/MIPS_Assembly)
5. [MARS - Mips Assembly and Runtime Simulator Help](https://dpetersanderson.github.io/Help/MarsHelpIntro.html)
6. [MIPS Reference Sheet - David Broman, KTH](https://www.kth.se/social/files/563c63c9f276547044e8695f/mips-ref-sheet.pdf)
7. [MIPS Instruction Set - Andrea Gasparetto](https://www.dsi.unive.it/~gasparetto/materials/MIPS_Instruction_Set.pdf)
8. [MIPS Architecture and Assembly Language Overview](https://minnie.tuhs.org/CompArch/Resources/mips_quick_tutorial.html)
9. [Accessing Memory in MIPS - Ziad Matni](https://ucsb-cs64.github.io/w20/lectures/lect07.pdf)
10. [MARS Memory-Mapped Input/Output](https://wilkinsonj.people.charleston.edu/mmio.html)
11. [Introduction to exceptions and interrupts in Mips](https://www2.it.uu.se/edu/course/homepage/os/vt20/module-1/assignment/)
12. [Input / Output (I/O) - Michael Langer](https://www.cim.mcgill.ca/~langer/273/20-slides.pdf)

### Memory

Check out the references used:

1. [MIPS Memory Map 1 - mips.com](https://training.mips.com/basic_mips/PDF/Memory_Map.pdf)
2. [MIPS Memory Map 2 - charleston.edu](https://wilkinsonj.people.charleston.edu/mem-map.html)
3. [MIPS memory layout - Uppsala](https://www.it.uu.se/education/course/homepage/os/vt18/module-0/mips-and-mars/mips-memory-layout/)
4. [Memory Management Unit - Wikipedia](https://en.wikipedia.org/wiki/Memory_management_unit)
5. [Memory Management - Wikipedia](https://en.wikipedia.org/wiki/Memory_management)
