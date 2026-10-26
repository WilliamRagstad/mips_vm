use std::{
    io::Write,
    process::{Command, Stdio},
};
fn invoke(mode: &str, file: &str, input: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mips_cli"))
        .args([mode, &format!("../examples/{file}.asm")])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
#[test]
fn examples_run_without_debugger() {
    for (file, input, expected) in [
        ("hello_world", "", "Hello, world!\n"),
        ("mem_text", "", "76"),
        (
            "mmio_hello_world",
            "X",
            "\nStart entering characters in the MMIO window.\nX",
        ),
        ("debug", "", ""),
    ] {
        let result = invoke("run", file, input);
        assert!(result.status.success(), "{file}: {:?}", result);
        assert_eq!(String::from_utf8(result.stdout).unwrap(), expected);
    }
}
#[test]
fn walkthrough() {
    let result=invoke("debug","debug","break loop\ncontinue\nstep\nmemory value 4\nregisters\ncontinue\ndelete loop\ncontinue\nquit\n");
    assert!(result.status.success());
    let s = String::from_utf8(result.stdout).unwrap();
    assert!(s.contains("0x10010000: 02 00 00 00"));
    assert!(s.contains("$t0   0x00000002"));
    assert!(s.contains("PC 0x0040001C halted"));
}
