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
