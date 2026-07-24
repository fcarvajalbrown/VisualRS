; Visual Rust — Inno Setup script (FUTURE / POST-MVP TEMPLATE)
;
; Status: NOT yet buildable. This is a template stowed for later, adapted from
; the RadarCL project's installer workflow. It packages the standalone egui
; editor binary (`vr-editor.exe`, ADR-0009) into a Windows installer. That binary
; does not exist yet — this script cannot compile until the editor ships and a
; release `cargo build --release` produces `target\release\vr-editor.exe`.
;
; Before this can be used, resolve every `TODO:` below. Items marked TODO are
; deliberately left blank rather than guessed — do not invent a publisher name,
; GUID, license, or icon.
;
; Requirements (when the time comes):
;   - Inno Setup 6.x (https://jrsoftware.org/isinfo.php)
;   - A release build: `cargo build --release -p vr-editor`
;     (produces target\release\vr-editor.exe)
;
; Build:
;   Open this file in the Inno Setup Compiler and click Build > Compile,
;   or: iscc packaging\windows\visual-rust.iss
;   Output: packaging\windows\output\VisualRust-Setup.exe

#define AppName "Visual Rust"
; TODO: set the real release version (keep in sync with the workspace Cargo.toml).
#define AppVersion "0.0.0"
; TODO: publisher name — do NOT guess. Fill in before release.
#define AppPublisher "TODO: publisher"
#define AppURL "https://github.com/fcarvajalbrown/VisualRS"
#define AppExeName "vr-editor.exe"
#define AppDescription "Node editor for visual programming in Rust: compiles a Blueprints-style graph to native Rust source"

[Setup]
; TODO: generate ONE stable, unique GUID and paste it here (Inno Setup menu:
; Tools > Generate GUID). It must never change across releases — it is how
; Windows recognizes upgrades of the same app. The value below is a placeholder
; and must be replaced.
AppId={{TODO-GENERATE-A-STABLE-GUID}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} v{#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}
AppUpdatesURL={#AppURL}
AppComments={#AppDescription}

; All Source/OutputDir paths below are relative to the repository root.
SourceDir=..\..
OutputDir=packaging\windows\output
OutputBaseFilename=VisualRust-Setup

; Installation directory
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
AllowNoIcons=yes

; TODO: provide a Windows .ico (e.g. produced from assets\logo.png) and point
; SetupIconFile + the [Icons] IconFilename entries at it. Left unset until then.
; SetupIconFile=assets\icon.ico
UninstallDisplayIcon={app}\{#AppExeName}

; Compression
Compression=lzma2/ultra64
SolidCompression=yes
LZMAUseSeparateProcess=yes

; Appearance
WizardStyle=modern
WizardResizable=yes
ShowLanguageDialog=no
LanguageDetectionMethod=none

; Windows version requirement (Windows 10+)
MinVersion=10.0

; Privileges — install per-user by default, allow elevation to all-users.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

; TODO: if/when the project has a LICENSE file, surface it in the wizard:
; LicenseFile=LICENSE

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon";   Description: "Create a &desktop shortcut";    GroupDescription: "Additional icons:"; Flags: unchecked
Name: "startmenuicon"; Description: "Create a &Start Menu shortcut"; GroupDescription: "Additional icons:";

[Files]
; Main executable — the standalone egui editor (ADR-0009).
Source: "target\release\{#AppExeName}"; DestDir: "{app}"; Flags: ignoreversion

; TODO: bundle the app icon once assets\icon.ico exists.
; Source: "assets\icon.ico"; DestDir: "{app}\assets"; Flags: ignoreversion

; Readme
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion isreadme

[Icons]
; Start Menu
Name: "{group}\{#AppName}";           Filename: "{app}\{#AppExeName}"
Name: "{group}\Uninstall {#AppName}"; Filename: "{uninstallexe}"

; Desktop (optional)
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Run]
; Offer to launch after install
Filename: "{app}\{#AppExeName}"; \
    Description: "Launch {#AppName} now"; \
    Flags: nowait postinstall skipifsilent

; TODO: if the editor writes user data (e.g. settings), add an [UninstallDelete]
; entry to clean it up on uninstall, once that path is defined.

[Messages]
WelcomeLabel2=This will install [name/ver] on your computer.%n%nVisual Rust is a node editor that compiles a Blueprints-style graph to real, idiomatic Rust source.%n%nClick Next to continue.
FinishedLabel=Visual Rust has been installed successfully.%n%nYou can find it in your Start Menu or launch it now using the checkbox below.
