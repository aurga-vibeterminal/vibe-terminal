//! Shared build-script helpers for the shipped binaries: stamping the git
//! commit into `VTERM_COMMIT_HASH` and its timestamp into `VTERM_COMMIT_TIME`,
//! and embedding the Windows VERSIONINFO resource (version, description,
//! copyright).

use std::env;
use std::path::PathBuf;
use std::process::Command;

/// LegalCopyright stamped into every shipped binary's version resource.
pub const COPYRIGHT: &str = "Copyright © 2026 AURGA CO., LIMITED";

/// Stamp the current git commit and its committer timestamp for startup logging.
pub fn emit_commit_hash() {
    let hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=VTERM_COMMIT_HASH={hash}");

    let commit_time = Command::new("git")
        .args(["show", "-s", "--format=%cI", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=VTERM_COMMIT_TIME={commit_time}");

    // Once any rerun-if-changed is emitted, only the listed paths trigger a
    // rebuild — watch HEAD and the branch ref it points at so a new commit
    // re-stamps the hash and timestamp.
    if let Ok(o) = Command::new("git")
        .args(["rev-parse", "--absolute-git-dir"])
        .output()
    {
        if o.status.success() {
            let git_dir = String::from_utf8_lossy(&o.stdout).trim().to_string();
            println!("cargo:rerun-if-changed={git_dir}/HEAD");
            if let Ok(head) = std::fs::read_to_string(format!("{git_dir}/HEAD")) {
                if let Some(reference) = head.trim().strip_prefix("ref: ") {
                    println!("cargo:rerun-if-changed={git_dir}/{reference}");
                }
            }
        }
    }
}

/// On Windows targets, embed a VERSIONINFO resource carrying the crate
/// version, `file_description`, product name, and COPYRIGHT. No-op on other
/// targets.
pub fn embed_windows_version_info(file_description: &str) {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    compile_rc(&version_info_rc(file_description));
}

/// VERSIONINFO .rc text for the calling crate. Crates that need extra
/// resources (e.g. an icon — link.exe honors only one resource input, so it
/// must share the .res) can prepend their own statements and pass the result
/// to [`compile_rc`].
///
/// The `#pragma code_page(65001)` makes the resource compiler read the file
/// as UTF-8 so the © in COPYRIGHT comes through — without it rc.exe decodes
/// as ANSI and produces mojibake.
pub fn version_info_rc(file_description: &str) -> String {
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let mut parts: Vec<&str> = version.split('.').collect();
    while parts.len() < 4 {
        parts.push("0");
    }
    let display_version = parts.join(".");
    format!(
        r#"#pragma code_page(65001)

1 VERSIONINFO
FILEVERSION {a},{b},{c},{d}
PRODUCTVERSION {a},{b},{c},{d}
FILEFLAGSMASK 0x3f
FILEFLAGS 0
FILEOS 0x40004
FILETYPE 1
FILESUBTYPE 0
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904b0"
        BEGIN
            VALUE "FileDescription", "{file_description}\0"
            VALUE "FileVersion", "{display_version}\0"
            VALUE "CompanyName", "AURGA CO., LIMITED\0"
            VALUE "LegalCopyright", "{COPYRIGHT}\0"
            VALUE "ProductName", "AURGA Vibe Terminal\0"
            VALUE "ProductVersion", "{display_version}\0"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x409, 1200
    END
END
"#,
        a = parts[0],
        b = parts[1],
        c = parts[2],
        d = parts[3],
    )
}

/// Write `rc` to OUT_DIR, compile it, and emit the link-arg. Prefers the
/// MSVC resource compiler when building for an msvc target: rc.exe is a HOST
/// tool whose .res output is architecture-neutral, so the same host rc.exe
/// serves every target arch. MinGW's windres always emits an x64 COFF
/// wrapper, which fails the ARM64 link with LNK1112 (module machine type
/// 'x64' conflicts with 'ARM64').
///
/// Failures only warn — the binary then ships without embedded resources.
pub fn compile_rc(rc: &str) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let rc_file = out_dir.join("version.rc");
    let res_file = out_dir.join("version.res");
    std::fs::write(&rc_file, rc).expect("write generated .rc");

    let target = env::var("TARGET").unwrap_or_default();
    let rc_exe = if target.contains("msvc") {
        // SDK per-arch rc.exe builds run on that arch only — pick the HOST
        // arch directory (std::env::consts::ARCH is the build script's own
        // architecture, i.e. the host), never the target's (the SDK's
        // bin\<ver>\arm64\rc.exe is an ARM64-native binary that cannot
        // run on an x64 cross-build host).
        let host_dir = match env::consts::ARCH {
            "x86_64" => "x64",
            "aarch64" => "arm64",
            "x86" => "x86",
            _ => "x64",
        };
        find_msvc_rc(host_dir)
    } else {
        None
    };

    if let Some(rc_exe) = rc_exe {
        let status = Command::new(&rc_exe)
            .arg("/nologo")
            .arg(format!("/fo{}", res_file.to_string_lossy()))
            .arg(&rc_file)
            .status();
        if matches!(status, Ok(s) if s.success()) {
            println!(
                "cargo:rustc-link-arg={}",
                res_file.to_string_lossy().replace("\\", "/")
            );
            return;
        }
    }

    // Fall back to MinGW's windres. `-c 65001` tells it to read the .rc as
    // UTF-8 (the code_page pragma covers rc.exe; windres needs the flag).
    let result = Command::new("windres")
        .env("PATH", "C:/msys64/mingw64/bin")
        .args([
            "-i",
            &rc_file.to_string_lossy().replace("\\", "/"),
            "-c",
            "65001",
            "-O",
            "coff",
            "-o",
            res_file.to_str().unwrap(),
        ])
        .status();

    match result {
        Ok(status) if status.success() => {
            println!(
                "cargo:rustc-link-arg={}",
                res_file.to_string_lossy().replace("\\", "/")
            );
        }
        _ => {
            eprintln!(
                "warning: could not compile Windows version resource \
                 (rc.exe/windres unavailable or failed); the binary ships \
                 without embedded version info"
            );
        }
    }
}

/// Locate the Windows SDK rc.exe for the given arch dir (x64/x86/arm64):
/// `%WindowsSdkDir%bin\<highest version>\<arch>\rc.exe`, falling back to
/// the standard `C:\Program Files (x86)\Windows Kits\10` location when the
/// variable is absent (e.g. no vcvars environment).
fn find_msvc_rc(arch_dir: &str) -> Option<PathBuf> {
    let sdk_dir = env::var_os("WindowsSdkDir")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10"));
    let bin = sdk_dir.join("bin");
    let mut best: Option<PathBuf> = None;
    let entries = std::fs::read_dir(&bin).ok()?;
    for entry in entries.flatten() {
        let rc = entry.path().join(arch_dir).join("rc.exe");
        if rc.exists() {
            best = Some(rc); // read_dir order is arbitrary; any hit works, keep last seen
        }
    }
    best
}
