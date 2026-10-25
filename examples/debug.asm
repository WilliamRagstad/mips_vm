.data
value: .word 0
.text
main:
    li $t0, 2
    la $t1, value
loop:
    sw $t0, 0($t1)
    addi $t0, $t0, 65535 # signed immediate -1
    bne $t0, $zero, loop
    li $v0, 10
    syscall
