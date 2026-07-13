//! Generated-code validity: the capstone must compile with the active `rustc`
//! and run correctly. Under CI's stable/beta/nightly matrix, this is the
//! three-channel validity check required by ADR-0007. Std-only, no network.

use std::process::Command;

/// A unique-enough temp subdir without external crates. Uses PID + a counter
/// baked into the path so parallel test binaries do not collide.
fn temp_dir(tag: &str) -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("vr_rustgen_{}_{}", std::process::id(), tag));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into())
}

#[test]
fn capstone_compiles_with_active_rustc() {
    let src = vr_rustgen::generate(&vr_ir::fixtures::line_report()).unwrap();
    let dir = temp_dir("compile");
    let src_path = dir.join("main.rs");
    std::fs::write(&src_path, &src).unwrap();

    // Type/borrow-check without producing a binary: fast validity gate.
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
fn capstone_runs_and_reports_counts() {
    let src = vr_rustgen::generate(&vr_ir::fixtures::line_report()).unwrap();
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

    // Sample input: 1 blank, 1 comment, 2 content lines, 5 content words.
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
