# MIPS VM

This is a simple MIPS32 (I) virtual machine interpreting assembly.

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
