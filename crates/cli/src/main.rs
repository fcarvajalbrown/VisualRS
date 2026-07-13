//! `vrc`: run the hardcoded Phase 1 capstone through the pipeline
//! (IR -> Rust source) and emit it. `--out <path>` writes to a file;
//! otherwise the source goes to stdout.

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut out_path: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => match args.next() {
                Some(p) => out_path = Some(p),
                None => {
                    eprintln!("error: --out requires a path");
                    return ExitCode::from(2);
                }
            },
            "-h" | "--help" => {
                println!("usage: vrc [--out <path>]");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("error: unknown argument `{other}`");
                return ExitCode::from(2);
            }
        }
    }

    let program = vr_ir::fixtures::line_report();
    let source = match vr_rustgen::generate(&program) {
        Ok(src) => src,
        Err(e) => {
            eprintln!("codegen error: {e}");
            return ExitCode::FAILURE;
        }
    };

    match out_path {
        Some(path) => {
            if let Err(e) = std::fs::write(&path, source) {
                eprintln!("error writing {path}: {e}");
                return ExitCode::FAILURE;
            }
        }
        None => print!("{source}"),
    }
    ExitCode::SUCCESS
}
