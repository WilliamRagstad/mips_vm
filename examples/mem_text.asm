.text
	addi $a1, $a0, 76
	addi $a2, $a0, 128000000
	sw $a1, 0($a2)
	lw $a3, 0($a2)

	# Print word at register $a3
	li $v0, 1
	move $a0, $a3
	syscall
