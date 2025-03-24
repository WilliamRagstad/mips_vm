.text
.globl echo
.globl main

main:
	li $v0, 4 # syscall 4 (print_str)
	la $a0, msg1
	syscall

echo:
	jal Read  # read and write using MMIO
	add $a0, $v0, $zero # infinite loop
	jal Write
	j echo

Read:
	lui $t0, 0xffff # MMIO base address
Loop1:
	lw $t1, 0($t0) # control
	andi $t1, $t1, 0x0001
	beq $t1, $zero, Loop1
	lw $v0, 4($t0) # data
	jr $ra

Write:
	lui $t0, 0xffff
Loop2:
	lw $t1, 8($t0) # control
	andi $t1, $t1, 0x0001
	beq $t1, $zero, Loop2
	sw $a0, 12($t0) # data
	jr $ra

.data
msg1: .asciiz "\nStart entering characters in the MMIO window.\n"
msg2: .asciiz "You entered: "
