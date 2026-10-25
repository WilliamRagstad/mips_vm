use mips_vm::{
    address::Address,
    parser::parse,
    registers::Register as R,
    vm::{Step, VM},
};
fn vm(code: &str) -> VM {
    VM::new(parse(code).unwrap(), vec![])
}
#[test]
fn registers_memory_pc_and_exit() {
    let mut v = vm(".data\nx: .word 0\n.text\nmain:\nli $t0, 7\nli $t1, 5\nadd $t2, $t0, $t1\nla $t3, x\nsw $t2, 0($t3)\nlw $t4, 0($t3)\nli $zero, 9\nli $v0, 10\nsyscall\n");
    for n in 1..=8 {
        assert_eq!(v.step().unwrap(), Step::Running);
        assert_eq!(v.pc().unwrap(), 0x400000 + n * 4);
    }
    assert_eq!(v.registers().get(&R::T2), 12);
    assert_eq!(v.registers().get(&R::T4), 12);
    assert_eq!(v.registers().get(&R::Zero), 0);
    assert_eq!(
        v.inspect(Address::new(0x10010000), 4).unwrap(),
        [12, 0, 0, 0]
    );
    assert_eq!(v.step().unwrap(), Step::Halted);
    let pc = v.pc();
    assert_eq!(v.step().unwrap(), Step::Halted);
    assert_eq!(v.pc(), pc);
}
#[test]
fn taken_untaken_and_backward_branches() {
    let mut v=vm(".text\nmain:\nli $t0, 2\nloop:\naddi $t0, $t0, 65535\nbne $t0, $zero, loop\nbeq $t0, $zero, done\nli $t1, 99\ndone:\nnop\n");
    let pcs = [4, 8, 4, 8, 12, 20, 24];
    for pc in pcs {
        v.step().unwrap();
        assert_eq!(v.pc().unwrap(), 0x400000 + pc);
    }
    assert_eq!(v.registers().get(&R::T0), 0);
    assert_eq!(v.registers().get(&R::T1), 0);
    assert_eq!(v.step().unwrap(), Step::Halted);
}
#[test]
fn call_and_return_without_delay_slot() {
    let mut v = vm(".text\nmain:\njal fun\nj done\nfun:\nli $t0, 7\njr $ra\ndone:\nnop\n");
    v.step().unwrap();
    assert_eq!(v.pc().unwrap(), 0x400008);
    assert_eq!(v.registers().get(&R::Ra), 0x400004);
    v.step().unwrap();
    v.step().unwrap();
    assert_eq!(v.pc().unwrap(), 0x400004);
    v.step().unwrap();
    assert_eq!(v.pc().unwrap(), 0x400010);
}
#[test]
fn faults_preserve_pc_and_registers() {
    for instruction in ["lw $t0, 0($zero)", "sw $t0, 0($zero)", "jr $t0"] {
        let mut v = vm(&format!(".text\nmain:\n{instruction}\n"));
        let pc = v.pc();
        assert!(v.step().is_err());
        assert_eq!(v.pc(), pc);
        assert_eq!(v.registers().get(&R::T0), 0);
    }
    let v = vm(".text\nmain:\nnop\n");
    for (addr, len) in [(0, 4), (0x400000, 257), (0x400000, 0), (u32::MAX, 4)] {
        assert!(v.inspect(Address::new(addr), len).is_err());
    }
}
#[test]
fn signed_comparison_shift_and_wrapping() {
    let mut v=vm(".text\nmain:\nlui $t0, 65535\nori $t0, $t0, 65535\nli $t1, 1\nslt $t2, $t0, $t1\nsltu $t3, $t0, $t1\nsra $t4, $t0, 4\naddu $t5, $t0, $t1\n");
    v.execute(v.entrypoint().unwrap()).unwrap();
    for (r, n) in [(R::T2, 1), (R::T3, 0), (R::T4, u32::MAX), (R::T5, 0)] {
        assert_eq!(v.registers().get(&r), n);
    }
}
#[test]
fn page_boundary_data_is_not_repeated() {
    let data = (0..4200)
        .map(|i| (b'A' + (i % 26) as u8) as char)
        .collect::<String>();
    let v = vm(&format!(".data\nx: .ascii \"{data}\"\n.text\nmain:\nnop\n"));
    assert_eq!(
        v.inspect(Address::new(0x10010000 + 4090), 20).unwrap(),
        data.as_bytes()[4090..4110]
    );
}
#[test]
fn arithmetic_and_logical_table() {
    for (op, y, want) in [
        ("sub", 3, 3),
        ("subu", 3, 3),
        ("mul", 3, 18),
        ("and", 3, 2),
        ("or", 3, 7),
        ("xor", 3, 5),
        ("nor", 3, !7u32),
        ("sllv", 33, 12),
        ("srlv", 33, 3),
    ] {
        let mut v = vm(&format!(
            ".text\nmain:\nli $t0, 6\nli $t1, {y}\n{op} $t2, $t0, $t1\n"
        ));
        v.execute(v.entrypoint().unwrap()).unwrap();
        assert_eq!(v.registers().get(&R::T2), want, "{op}");
        assert_eq!(v.pc().unwrap(), 0x40000c);
    }
    for (op, want) in [
        ("andi", 2),
        ("ori", 7),
        ("xori", 5),
        ("sll", 48),
        ("srl", 0),
        ("slti", 0),
        ("sltiu", 0),
        ("addiu", 9),
    ] {
        let mut v = vm(&format!(".text\nmain:\nli $t0, 6\n{op} $t1, $t0, 3\n"));
        v.execute(v.entrypoint().unwrap()).unwrap();
        assert_eq!(v.registers().get(&R::T1), want, "{op}");
    }
}
#[test]
fn byte_extension_overflow_and_unsupported_syscalls() {
    let mut v=vm(".data\nx: .word 0\n.text\nmain:\nla $t0, x\nli $t1, 255\nsb $t1, 0($t0)\nlb $t2, 0($t0)\nlbu $t3, 0($t0)\n");
    v.execute(v.entrypoint().unwrap()).unwrap();
    assert_eq!(v.registers().get(&R::T2), u32::MAX);
    assert_eq!(v.registers().get(&R::T3), 255);
    let mut v = vm(".text\nmain:\nlui $t0, 32767\nori $t0, $t0, 65535\naddi $t1, $t0, 1\n");
    v.step().unwrap();
    v.step().unwrap();
    assert!(v.step().is_err());
    assert_eq!(v.pc().unwrap(), 0x400008);
    assert_eq!(v.registers().get(&R::T1), 0);
    let mut v = vm(".text\nmain:\nli $v0, 2\nsyscall\n");
    v.step().unwrap();
    assert!(v.step().is_err());
}
#[test]
fn untaken_beq_jalr_alignment_and_heap() {
    let mut v=vm(".text\nmain:\nli $t0, 1\nbeq $t0, $zero, fun\nla $t1, fun\njalr $ra, $t1\nj done\nfun:\njr $ra\ndone:\nnop\n");
    v.step().unwrap();
    v.step().unwrap();
    assert_eq!(v.pc().unwrap(), 0x400008);
    v.step().unwrap();
    v.step().unwrap();
    assert_eq!(v.pc().unwrap(), 0x400014);
    assert_eq!(v.registers().get(&R::Ra), 0x400010);
    v.step().unwrap();
    assert_eq!(v.pc().unwrap(), 0x400010);
    let mut v = vm(".data\nx: .word 0\n.text\nmain:\nla $t0, x\nlw $t1, 1($t0)\n");
    v.step().unwrap();
    assert!(v.step().is_err());
    assert_eq!(v.pc().unwrap(), 0x400004);
    let mut v = vm(
        ".text\nmain:\nli $a0, 4\nli $v0, 9\nsyscall\nli $t0, 42\nsw $t0, 0($v0)\nlw $t1, 0($v0)\n",
    );
    v.execute(v.entrypoint().unwrap()).unwrap();
    assert_eq!(v.registers().get(&R::T1), 42);
}
