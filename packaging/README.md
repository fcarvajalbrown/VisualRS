# Packaging

Distribution artifacts for Visual Rust. These are **future / post-MVP** templates,
stowed ahead of need; none of them can be built until the standalone editor
binary (`vr-editor.exe`, [ADR-0009](../docs/adr/0009-editor-host-standalone-egui-app.md))
exists.

## Windows — `windows/visual-rust.iss`

An [Inno Setup](https://jrsoftware.org/isinfo.php) 6.x script that packages the
release editor binary into a Windows installer. Adapted from the RadarCL project's
installer workflow.

- **Prerequisite:** a release build — `cargo build --release -p vr-editor`,
  producing `target\release\vr-editor.exe`.
- **Build:** open `windows/visual-rust.iss` in the Inno Setup Compiler and
  Build > Compile, or run `iscc packaging\windows\visual-rust.iss`.
- **Output:** `packaging/windows/output/VisualRust-Setup.exe`.
- **Before first use, resolve every `TODO:` in the script** — publisher name, a
  stable `AppId` GUID, the app `.ico`, the release version, and (if adopted) the
  license file. Those are left blank on purpose rather than filled with guesses.

macOS and Linux packaging are not templated yet; the editor targets all three
(ADR-0009), so they will be added when the binary and a release process exist.
