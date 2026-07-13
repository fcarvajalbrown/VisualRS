//! Generated-code validity from the graph front-end: lower the capstone graph,
//! generate Rust, compile it with the active `rustc`, and run it against a
//! sample input. Std-only, no network. Mirrors the rustgen compile test.

use std::process::Command;

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("vr_graph_{}_{}", std::process::id(), tag));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into())
}

fn generate() -> String {
    let prog = vr_graph::lower(&vr_graph::fixtures::line_report_graph()).unwrap();
    vr_rustgen::generate(&prog).unwrap()
}

#[test]
fn graph_capstone_compiles_with_active_rustc() {
    let src = generate();
    let dir = temp_dir("compile");
    let src_path = dir.join("main.rs");
    std::fs::write(&src_path, &src).unwrap();

    let out = Command::new(rustc())
        .args([
            "--edition",
            "2021",
            "--crate-type",
            "bin",
            "--emit=metadata",
        ])
        .arg(&src_path)
        .arg("--out-dir")
        .arg(&dir)
        .output()
        .expect("failed to invoke rustc");

    assert!(
        out.status.success(),
        "generated code failed to compile:\n--- source ---\n{src}\n--- stderr ---\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn graph_capstone_runs_and_reports_counts() {
    let src = generate();
    let dir = temp_dir("run");
    let src_path = dir.join("main.rs");
    std::fs::write(&src_path, &src).unwrap();

    let exe = dir.join(if cfg!(windows) {
        "report.exe"
    } else {
        "report"
    });
    let build = Command::new(rustc())
        .args(["--edition", "2021", "-O"])
        .arg(&src_path)
        .arg("-o")
        .arg(&exe)
        .output()
        .expect("failed to invoke rustc");
    assert!(
        build.status.success(),
        "build failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );

    let input = dir.join("sample.txt");
    std::fs::write(&input, "\n# a comment\nhello world\nthree more words\n").unwrap();

    let run = Command::new(&exe).arg(&input).output().expect("run failed");
    assert!(run.status.success(), "program exited with error: {run:?}");
    let stdout = String::from_utf8_lossy(&run.stdout);

    assert!(stdout.contains("lines: 4"), "got:\n{stdout}");
    assert!(stdout.contains("blank: 1"), "got:\n{stdout}");
    assert!(stdout.contains("comment: 1"), "got:\n{stdout}");
    assert!(stdout.contains("content: 2"), "got:\n{stdout}");
    assert!(stdout.contains("words: 5"), "got:\n{stdout}");
}
