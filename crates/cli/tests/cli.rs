use std::process::Command;

fn vrc() -> Command {
    // Cargo sets CARGO_BIN_EXE_<name> for integration tests.
    Command::new(env!("CARGO_BIN_EXE_vrc"))
}

#[test]
fn prints_generated_source_to_stdout() {
    let out = vrc().output().expect("run vrc");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("fn main()"), "got:\n{stdout}");
    assert!(stdout.contains("struct Report"), "got:\n{stdout}");
}

#[test]
fn writes_to_out_path() {
    let dir = std::env::temp_dir().join(format!("vrc_out_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let out_path = dir.join("generated.rs");

    let status = vrc()
        .arg("--out")
        .arg(&out_path)
        .status()
        .expect("run vrc --out");
    assert!(status.success());

    let contents = std::fs::read_to_string(&out_path).unwrap();
    assert!(contents.contains("fn build_report"), "got:\n{contents}");
}
