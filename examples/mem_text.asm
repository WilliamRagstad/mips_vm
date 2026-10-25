.data
value: .word 0
.text
main:
    li $a1, 76
    la $a2, value
    sw $a1, 0($a2)
    lw $a3, 0($a2)
    li $v0, 1
    move $a0, $a3
    syscall
    li $v0, 10
    syscall
