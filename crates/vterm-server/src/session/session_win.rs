use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use vterm_common::TerminalProfile;

use super::terminal_profiles::{default_shell, LaunchProfile};

pub(super) fn available_terminal_profiles() -> Vec<TerminalProfile> {
    let from_settings = windows_terminal_profiles_from_settings();
    let mut profiles = if from_settings.is_empty() {
        windows_fallback_profiles()
    } else {
        from_settings
    };

    append_missing_profiles(
        &mut profiles,
        msys2_profile().into_iter().chain(git_bash_profile()),
    );
    profiles
}

fn append_missing_profiles(
    profiles: &mut Vec<TerminalProfile>,
    discovered: impl IntoIterator<Item = TerminalProfile>,
) {
    for profile in discovered {
        if !profiles
            .iter()
            .any(|existing| same_terminal_profile(existing, &profile))
        {
            profiles.push(profile);
        }
    }
}

fn same_terminal_profile(left: &TerminalProfile, right: &TerminalProfile) -> bool {
    left.title.eq_ignore_ascii_case(&right.title)
        || left
            .program
            .replace('/', "\\")
            .eq_ignore_ascii_case(&right.program.replace('/', "\\"))
}

fn windows_terminal_profiles_from_settings() -> Vec<TerminalProfile> {
    let Some(settings_path) = windows_terminal_settings_path() else {
        return Vec::new();
    };
    let Ok(contents) = std::fs::read_to_string(settings_path) else {
        return Vec::new();
    };

    parse_windows_terminal_settings(&contents)
}

fn windows_terminal_local_state_dir() -> Option<PathBuf> {
    let local_app_data = std::env::var_os("LOCALAPPDATA")?;
    Some(
        PathBuf::from(local_app_data)
            .join("Packages")
            .join("Microsoft.WindowsTerminal_8wekyb3d8bbwe")
            .join("LocalState"),
    )
}

fn windows_terminal_settings_path() -> Option<PathBuf> {
    let settings_path = windows_terminal_local_state_dir()?.join("settings.json");
    settings_path.exists().then_some(settings_path)
}

fn parse_windows_terminal_settings(contents: &str) -> Vec<TerminalProfile> {
    let Ok(settings) = serde_json::from_str::<Value>(contents) else {
        return Vec::new();
    };

    let default_profile = settings
        .get("defaultProfile")
        .and_then(Value::as_str)
        .map(str::to_string);

    let Some(profile_list) = settings
        .get("profiles")
        .and_then(|profiles| profiles.get("list"))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };

    let mut profiles = Vec::new();
    for profile in profile_list {
        if let Some(profile) = parse_windows_terminal_profile(profile, default_profile.as_deref()) {
            profiles.push(profile);
        }
    }

    profiles
}

fn parse_windows_terminal_profile(
    profile: &Value,
    default_profile: Option<&str>,
) -> Option<TerminalProfile> {
    if profile
        .get("hidden")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return None;
    }

    let title = profile
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())?
        .to_string();

    let id = profile
        .get("guid")
        .and_then(Value::as_str)
        .map(normalized_profile_id)
        .unwrap_or_else(|| normalized_profile_id(&title));

    let launch = if let Some(commandline) = profile.get("commandline").and_then(Value::as_str) {
        launch_profile_from_commandline(&title, commandline)?
    } else {
        launch_profile_from_windows_terminal_metadata(profile, &title)?
    };

    Some(TerminalProfile {
        id,
        title,
        program: launch.program,
        args: launch.args,
        is_default: profile
            .get("guid")
            .and_then(Value::as_str)
            .zip(default_profile)
            .map(|(guid, default_guid)| guid.eq_ignore_ascii_case(default_guid))
            .unwrap_or(false),
        server_id: None,
        server_name: None,
    })
}

fn launch_profile_from_commandline(title: &str, commandline: &str) -> Option<LaunchProfile> {
    let expanded = expand_windows_environment_variables(commandline);
    let (program, args) = split_windows_launch_commandline(&expanded)?;

    Some(LaunchProfile {
        title: title.to_string(),
        program,
        args,
    })
}

fn launch_profile_from_windows_terminal_metadata(
    profile: &Value,
    title: &str,
) -> Option<LaunchProfile> {
    let source = profile.get("source").and_then(Value::as_str)?;

    if is_wsl_profile_source(source) {
        let program = resolve_command_path("wsl.exe")?;
        let mut args = wsl_distribution_name(title, source)
            .map(|distro| vec!["-d".to_string(), distro])
            .unwrap_or_default();
        args.extend(wsl_default_directory_args());
        return Some(LaunchProfile {
            title: title.to_string(),
            program,
            args,
        });
    }

    if source.eq_ignore_ascii_case("Windows.Terminal.PowershellCore") {
        let program =
            resolve_command_path("pwsh.exe").or_else(|| resolve_command_path("powershell.exe"))?;
        return Some(LaunchProfile {
            title: title.to_string(),
            program,
            args: Vec::new(),
        });
    }

    if source.eq_ignore_ascii_case("Windows.Terminal.VisualStudio") {
        return visual_studio_launch_profile(title);
    }

    if source.to_ascii_lowercase().contains("anaconda") {
        return anaconda_powershell_profile().map(|profile| LaunchProfile {
            title: profile.title,
            program: profile.program,
            args: profile.args,
        });
    }

    None
}

fn split_windows_commandline(commandline: &str) -> Vec<String> {
    let mut arguments = Vec::new();
    let mut current = String::new();
    let mut characters = commandline.chars().peekable();
    let mut in_quotes = false;

    while let Some(character) = characters.next() {
        match character {
            '"' => {
                in_quotes = !in_quotes;
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    arguments.push(std::mem::take(&mut current));
                }
            }
            '\\' => {
                let mut slash_count = 1;
                while matches!(characters.peek(), Some('\\')) {
                    characters.next();
                    slash_count += 1;
                }

                if matches!(characters.peek(), Some('"')) {
                    for _ in 0..(slash_count / 2) {
                        current.push('\\');
                    }
                    if slash_count % 2 == 0 {
                        characters.next();
                        in_quotes = !in_quotes;
                    } else {
                        characters.next();
                        current.push('"');
                    }
                } else {
                    for _ in 0..slash_count {
                        current.push('\\');
                    }
                }
            }
            _ => current.push(character),
        }
    }

    if !current.is_empty() {
        arguments.push(current);
    }

    arguments
}

fn resolve_profile_program(program: &str) -> Option<String> {
    let expanded = expand_windows_environment_variables(program);
    let trimmed = expanded.trim();
    if trimmed.is_empty() {
        return None;
    }

    let path = Path::new(trimmed);
    if path.is_absolute() {
        return path.exists().then(|| trimmed.to_string());
    }

    resolve_command_path(trimmed)
        .or_else(|| resolve_command_path(&format!("{trimmed}.exe")))
        .or_else(|| Some(trimmed.to_string()))
}

fn split_windows_launch_commandline(commandline: &str) -> Option<(String, Vec<String>)> {
    let arguments = split_windows_commandline(commandline);
    if let Some((program, args)) = arguments.split_first() {
        if let Some(program) = resolve_profile_program(program) {
            return Some((program, args.to_vec()));
        }
    }

    split_windows_commandline_from_existing_path(commandline)
}

fn split_windows_commandline_from_existing_path(
    commandline: &str,
) -> Option<(String, Vec<String>)> {
    let trimmed = commandline.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(stripped) = trimmed.strip_prefix('"') {
        let closing_quote = stripped.find('"')?;
        let program = &stripped[..closing_quote];
        let program = resolve_profile_program(program)?;
        let args = split_windows_commandline(stripped[closing_quote + 1..].trim_start());
        return Some((program, args));
    }

    for extension in [".exe", ".cmd", ".bat", ".com"] {
        let mut search_start = 0;
        while let Some(relative_index) =
            trimmed[search_start..].to_ascii_lowercase().find(extension)
        {
            let end = search_start + relative_index + extension.len();
            let program = trimmed[..end].trim();
            if let Some(program) = resolve_profile_program(program) {
                let args = split_windows_commandline(trimmed[end..].trim_start());
                return Some((program, args));
            }
            search_start = end;
        }
    }

    None
}

fn windows_fallback_profiles() -> Vec<TerminalProfile> {
    let mut profiles = Vec::new();

    if let Some(program) = resolve_command_path("powershell.exe") {
        profiles.push(TerminalProfile {
            id: "windows-powershell".to_string(),
            title: "Windows PowerShell".to_string(),
            program,
            args: Vec::new(),
            is_default: true,
            server_id: None,
            server_name: None,
        });
    }

    if let Some(program) = resolve_command_path("cmd.exe").or_else(resolve_comspec_path) {
        profiles.push(TerminalProfile {
            id: "command-prompt".to_string(),
            title: "Command Prompt".to_string(),
            program,
            args: Vec::new(),
            is_default: profiles.is_empty(),
            server_id: None,
            server_name: None,
        });
    }

    if let Some(program) = resolve_command_path("pwsh.exe") {
        profiles.push(TerminalProfile {
            id: "powershell-7".to_string(),
            title: "PowerShell".to_string(),
            program,
            args: Vec::new(),
            is_default: false,
            server_id: None,
            server_name: None,
        });
    }

    if let Some(program) = resolve_command_path("wsl.exe") {
        let distros = wsl_distributions();
        if distros.is_empty() {
            profiles.push(TerminalProfile {
                id: "wsl".to_string(),
                title: "WSL".to_string(),
                program,
                args: wsl_default_directory_args(),
                is_default: false,
                server_id: None,
                server_name: None,
            });
        } else {
            for distro in distros {
                let id = format!("wsl-{}", normalized_profile_id(&distro));
                let title = format!("{} (WSL)", distro);
                let mut args = vec!["-d".to_string(), distro];
                args.extend(wsl_default_directory_args());
                profiles.push(TerminalProfile {
                    id,
                    title,
                    program: program.clone(),
                    args,
                    is_default: false,
                    server_id: None,
                    server_name: None,
                });
            }
        }
    }

    if let Some(profile) = anaconda_powershell_profile() {
        profiles.push(profile);
    }

    if profiles.is_empty() {
        let program = default_shell();
        profiles.push(TerminalProfile {
            id: "default-shell".to_string(),
            title: "Default Shell".to_string(),
            program,
            args: Vec::new(),
            is_default: true,
            server_id: None,
            server_name: None,
        });
    }

    profiles
}

fn resolve_comspec_path() -> Option<String> {
    std::env::var("COMSPEC")
        .ok()
        .filter(|path| Path::new(path).exists())
}

fn resolve_command_path(command: &str) -> Option<String> {
    let path = Path::new(command);
    if path.is_absolute() && path.exists() {
        return Some(command.to_string());
    }

    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .map(|directory| directory.join(command))
        .find(|candidate| candidate.exists())
        .map(|candidate| candidate.to_string_lossy().into_owned())
}

fn msys2_profile() -> Option<TerminalProfile> {
    let mut roots = Vec::new();
    if let Some(root) = std::env::var_os("MSYS2_ROOT") {
        roots.push(PathBuf::from(root));
    }
    roots.extend(
        [
            r"C:\msys64",
            r"C:\msys32",
            r"C:\tools\msys64",
            r"C:\tools\msys32",
        ]
        .into_iter()
        .map(PathBuf::from),
    );
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        roots.push(PathBuf::from(program_files).join("msys64"));
    }
    if let Some(launcher) = resolve_command_path("msys2_shell.cmd") {
        if let Some(root) = Path::new(&launcher).parent() {
            roots.push(root.to_path_buf());
        }
    }

    roots
        .into_iter()
        .find_map(|root| msys2_profile_from_root(&root))
}

fn msys2_profile_from_root(root: &Path) -> Option<TerminalProfile> {
    let program = root.join("usr").join("bin").join("bash.exe");
    program
        .is_file()
        .then(|| bash_profile("msys2", "MSYS2", program))
}

fn git_bash_profile() -> Option<TerminalProfile> {
    let git = resolve_command_path("git.exe")?;
    git_bash_profile_from_executable(Path::new(&git))
}

fn git_bash_profile_from_executable(git: &Path) -> Option<TerminalProfile> {
    git.ancestors().find_map(|root| {
        let is_git_root = root.join("cmd").join("git.exe").is_file()
            || root.join("mingw64").join("bin").join("git.exe").is_file()
            || root.join("git-bash.exe").is_file();
        if !is_git_root {
            return None;
        }

        let bin_bash = root.join("bin").join("bash.exe");
        let usr_bin_bash = root.join("usr").join("bin").join("bash.exe");
        [bin_bash, usr_bin_bash]
            .into_iter()
            .find(|program| program.is_file())
            .map(|program| bash_profile("git-bash", "Git Bash", program))
    })
}

fn bash_profile(id: &str, title: &str, program: PathBuf) -> TerminalProfile {
    TerminalProfile {
        id: id.to_string(),
        title: title.to_string(),
        program: program.to_string_lossy().into_owned(),
        args: vec!["--login".to_string(), "-i".to_string()],
        is_default: false,
        server_id: None,
        server_name: None,
    }
}

fn expand_windows_environment_variables(value: &str) -> String {
    let mut expanded = String::with_capacity(value.len());
    let mut characters = value.chars().peekable();

    while let Some(character) = characters.next() {
        if character != '%' {
            expanded.push(character);
            continue;
        }

        let mut variable = String::new();
        while let Some(next) = characters.next() {
            if next == '%' {
                break;
            }
            variable.push(next);
        }

        if variable.is_empty() {
            expanded.push('%');
            continue;
        }

        match std::env::var(&variable) {
            Ok(replacement) => expanded.push_str(&replacement),
            Err(_) => {
                expanded.push('%');
                expanded.push_str(&variable);
                expanded.push('%');
            }
        }
    }

    expanded
}

fn wsl_distributions() -> Vec<String> {
    let mut command = Command::new("wsl.exe");
    command.args(["-l", "-q"]);
    vterm_common::command_no_window(&mut command);
    let output = match command.output() {
        Ok(output) if output.status.success() => output,
        _ => return Vec::new(),
    };

    parse_wsl_distributions(&output.stdout)
}

fn parse_wsl_distributions(stdout: &[u8]) -> Vec<String> {
    decode_windows_console_output(stdout)
        .lines()
        .map(|line| line.trim().trim_matches('\u{feff}'))
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect()
}

fn decode_windows_console_output(stdout: &[u8]) -> String {
    let looks_like_utf16 =
        stdout.len() >= 2 && stdout.chunks_exact(2).any(|chunk| chunk.get(1) == Some(&0));

    if looks_like_utf16 {
        let utf16_units = stdout
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();

        return String::from_utf16_lossy(&utf16_units);
    }

    String::from_utf8_lossy(stdout).into_owned()
}

fn is_wsl_profile_source(source: &str) -> bool {
    source.eq_ignore_ascii_case("Windows.Terminal.Wsl")
        || source.eq_ignore_ascii_case("Microsoft.WSL")
        || source.starts_with("CanonicalGroupLimited.")
}

fn wsl_distribution_name(title: &str, source: &str) -> Option<String> {
    let distros = wsl_distributions();
    if distros.is_empty() {
        return title
            .strip_suffix(" (WSL)")
            .map(|value| value.trim().to_string())
            .or_else(|| Some(title.trim().to_string()))
            .filter(|value| !value.is_empty());
    }

    let candidates = [
        title
            .strip_suffix(" (WSL)")
            .unwrap_or(title)
            .trim()
            .to_string(),
        source_package_display_name(source).unwrap_or_default(),
    ];

    candidates
        .into_iter()
        .filter(|candidate| !candidate.is_empty())
        .find_map(|candidate| best_matching_wsl_distribution(&distros, &candidate))
        .or_else(|| {
            distros
                .into_iter()
                .find(|distro| distro.eq_ignore_ascii_case(title.trim()))
        })
}

fn source_package_display_name(source: &str) -> Option<String> {
    let package_name = source
        .strip_prefix("CanonicalGroupLimited.")
        .unwrap_or(source)
        .split('_')
        .next()?
        .trim();
    (!package_name.is_empty()).then(|| package_name.replace("LTS", " LTS"))
}

fn best_matching_wsl_distribution(distros: &[String], candidate: &str) -> Option<String> {
    let normalized_candidate = normalized_profile_lookup(candidate);
    if normalized_candidate.is_empty() {
        return None;
    }

    distros
        .iter()
        .find(|distro| normalized_profile_lookup(distro) == normalized_candidate)
        .cloned()
        .or_else(|| {
            distros
                .iter()
                .find(|distro| {
                    let normalized_distro = normalized_profile_lookup(distro);
                    normalized_distro.contains(&normalized_candidate)
                        || normalized_candidate.contains(&normalized_distro)
                })
                .cloned()
        })
}

fn normalized_profile_lookup(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_lowercase())
        .collect::<String>()
        .replace("preview", "")
        .replace("lts", "")
}

fn wsl_default_directory_args() -> Vec<String> {
    vec!["--cd".to_string(), "~".to_string()]
}

fn normalized_profile_id(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

#[derive(Clone)]
struct VisualStudioInstallation {
    installation_path: PathBuf,
    product_line_version: Option<String>,
    is_prerelease: bool,
}

fn visual_studio_launch_profile(title: &str) -> Option<LaunchProfile> {
    let installation = best_matching_visual_studio_installation(title)?;
    let normalized_title = title.to_ascii_lowercase();

    if normalized_title.contains("powershell") {
        let program =
            resolve_command_path("powershell.exe").or_else(|| resolve_command_path("pwsh.exe"))?;
        let script_path = installation
            .installation_path
            .join("Common7")
            .join("Tools")
            .join("Launch-VsDevShell.ps1");
        if !script_path.exists() {
            return None;
        }

        // Without -Arch, Enter-VsDevShell defaults to the 32-bit x86 tools
        // even on a 64-bit host (verified: `cl` then reports "for x86"), so
        // pass the native architecture explicitly to match what Windows
        // Terminal's own Visual Studio profiles open. -Arch/-HostArch exist
        // only in the VS 17.1+ script — an older one (VS 2019) rejects the
        // unknown parameters outright, so append them only when declared.
        let mut command = format!("& '{}'", script_path.to_string_lossy().replace('\'', "''"));
        command.push_str(&powershell_dev_shell_arch_args(&script_path));

        return Some(LaunchProfile {
            title: title.to_string(),
            program,
            args: vec![
                "-NoExit".to_string(),
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-Command".to_string(),
                command,
            ],
        });
    }

    if normalized_title.contains("command prompt") {
        let program = resolve_command_path("cmd.exe").or_else(resolve_comspec_path)?;
        let script_path = installation
            .installation_path
            .join("Common7")
            .join("Tools")
            .join("VsDevCmd.bat");
        if !script_path.exists() {
            return None;
        }

        // Same 32-bit-by-default story as the PowerShell profile above;
        // VsDevCmd.bat has parsed -arch= since VS 2019 (in
        // vsdevcmd\core\parse_cmd.bat), so no capability check is needed.
        let mut args = vec!["/K".to_string(), script_path.to_string_lossy().into_owned()];
        if let Some(arch) = native_dev_shell_arch() {
            args.push(format!("-arch={arch}"));
            args.push(format!("-host_arch={arch}"));
        }

        return Some(LaunchProfile {
            title: title.to_string(),
            program,
            args,
        });
    }

    None
}

/// Native VS developer-shell target architecture of this machine: "x64" or
/// "arm64" on 64-bit Windows, `None` on 32-bit (the x86 default is already
/// correct there).
///
/// `PROCESSOR_ARCHITEW6432` reports the real OS architecture even from a
/// WOW64 process (`PROCESSOR_ARCHITECTURE` then carries the emulated one),
/// so it wins when present. An x64 client/server running on ARM64 Windows
/// thus sees ARM64 and selects the arm64 tools with an x64 host — matching
/// VS's own "arm64 native" prompt (VS itself runs as an x64 process there).
fn native_dev_shell_arch() -> Option<&'static str> {
    let architecture = std::env::var("PROCESSOR_ARCHITEW6432")
        .or_else(|_| std::env::var("PROCESSOR_ARCHITECTURE"))
        .ok()?
        .to_ascii_uppercase();
    if architecture.contains("ARM64") {
        Some("arm64")
    } else if architecture.contains("AMD64") {
        Some("x64")
    } else {
        None
    }
}

/// " -Arch <target> -HostArch <host>" suffix for the Launch-VsDevShell.ps1
/// invocation, or "" when the machine is 32-bit or the script predates the
/// parameters. The wrapper's param block validates `Arch` against
/// `x86|amd64|arm|arm64` — note `amd64`, not `x64` — and `HostArch` against
/// `x86|amd64`; VS runs as an x64 process even on ARM64, hence the x64 host
/// everywhere.
fn powershell_dev_shell_arch_args(script_path: &Path) -> String {
    let Some(arch) = native_dev_shell_arch() else {
        return String::new();
    };
    // VS 2019 / early VS 2022 scripts lack the parameters and would fail
    // parameter binding; detect them by their own param declarations.
    let declares_arch_params = std::fs::read_to_string(script_path)
        .map(|text| text.contains("$Arch") && text.contains("$HostArch"))
        .unwrap_or(false);
    if !declares_arch_params {
        return String::new();
    }
    let target = if arch == "arm64" { "arm64" } else { "amd64" };
    format!(" -Arch {target} -HostArch amd64")
}

fn best_matching_visual_studio_installation(title: &str) -> Option<VisualStudioInstallation> {
    let installs = visual_studio_installations();
    if installs.is_empty() {
        return None;
    }

    let normalized_title = title.to_ascii_lowercase();
    let wants_preview = normalized_title.contains("preview");
    let version_hint = if normalized_title.contains("2022") {
        Some("2022")
    } else if normalized_title.contains("vs 18") || normalized_title.contains("vs18") {
        Some("18")
    } else if normalized_title.contains("2019") {
        Some("2019")
    } else {
        None
    };

    installs
        .iter()
        .find(|install| visual_studio_installation_matches(install, version_hint, wants_preview))
        .or_else(|| {
            installs
                .iter()
                .find(|install| install.is_prerelease == wants_preview)
        })
        .or_else(|| installs.first())
        .cloned()
}

fn visual_studio_installation_matches(
    installation: &VisualStudioInstallation,
    version_hint: Option<&str>,
    wants_preview: bool,
) -> bool {
    if installation.is_prerelease != wants_preview {
        return false;
    }

    match version_hint {
        Some(version_hint) => installation
            .product_line_version
            .as_deref()
            .map(|version| version.eq_ignore_ascii_case(version_hint))
            .unwrap_or(false),
        None => true,
    }
}

fn visual_studio_installations() -> Vec<VisualStudioInstallation> {
    let Some(vswhere) = visual_studio_where_path() else {
        return Vec::new();
    };
    let mut command = Command::new(vswhere);
    command.args([
        "-format",
        "json",
        "-utf8",
        "-products",
        "*",
        "-prerelease",
        "-all",
    ]);
    vterm_common::command_no_window(&mut command);
    let output = match command.output() {
        Ok(output) if output.status.success() => output,
        _ => return Vec::new(),
    };

    let Ok(entries) = serde_json::from_slice::<Vec<Value>>(&output.stdout) else {
        return Vec::new();
    };

    entries
        .into_iter()
        .filter_map(|entry| {
            let installation_path = entry
                .get("installationPath")
                .and_then(Value::as_str)
                .map(PathBuf::from)?;
            let is_prerelease = entry
                .get("isPrerelease")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let product_line_version = entry
                .get("catalog")
                .and_then(|catalog| catalog.get("productLineVersion"))
                .and_then(Value::as_str)
                .map(str::to_string);

            Some(VisualStudioInstallation {
                installation_path,
                product_line_version,
                is_prerelease,
            })
        })
        .collect()
}

fn visual_studio_where_path() -> Option<PathBuf> {
    let program_files_x86 = std::env::var_os("ProgramFiles(x86)")?;
    let path = PathBuf::from(program_files_x86)
        .join("Microsoft Visual Studio")
        .join("Installer")
        .join("vswhere.exe");
    path.exists().then_some(path)
}

fn anaconda_powershell_profile() -> Option<TerminalProfile> {
    let conda_exe = conda_executable_path()?;
    let base_dir = conda_root_dir(&conda_exe)?;
    let hook = base_dir
        .join("shell")
        .join("condabin")
        .join("conda-hook.ps1");
    let powershell = resolve_command_path("powershell.exe")?;

    if !hook.exists() {
        return None;
    }

    let base_dir = base_dir.to_string_lossy().replace('\'', "''");
    let hook = hook.to_string_lossy().replace('\'', "''");
    let command = format!("& '{}' ; conda activate '{}'", hook, base_dir);

    Some(TerminalProfile {
        id: "anaconda-powershell".to_string(),
        title: "Anaconda PowerShell".to_string(),
        program: powershell,
        args: vec![
            "-NoExit".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-Command".to_string(),
            command,
        ],
        is_default: false,
        server_id: None,
        server_name: None,
    })
}

fn conda_executable_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CONDA_EXE").map(PathBuf::from) {
        if path.exists() {
            return Some(path);
        }
    }

    if let Some(path) = resolve_command_path("conda.exe") {
        return Some(PathBuf::from(path));
    }

    let user_profile = std::env::var_os("USERPROFILE").map(PathBuf::from)?;
    [
        user_profile
            .join("anaconda3")
            .join("Scripts")
            .join("conda.exe"),
        user_profile
            .join("miniconda3")
            .join("Scripts")
            .join("conda.exe"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn conda_root_dir(conda_exe: &Path) -> Option<PathBuf> {
    let parent = conda_exe.parent()?;
    let directory_name = parent.file_name()?.to_string_lossy().to_ascii_lowercase();
    if directory_name == "scripts" || directory_name == "condabin" {
        return parent.parent().map(Path::to_path_buf);
    }
    Some(parent.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_shell_arch_args_follow_script_capability() {
        let temp_dir = std::env::temp_dir().join("vibe dev shell arch tests");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Modern (VS 17.1+) wrapper declares the parameters; the VS 2019
        // one does not and must not receive them.
        let modern = temp_dir.join("modern.ps1");
        std::fs::write(&modern, "param($Arch, $HostArch)\n").unwrap();
        let legacy = temp_dir.join("legacy.ps1");
        std::fs::write(&legacy, "param($List)\n").unwrap();

        let legacy_args = powershell_dev_shell_arch_args(&legacy);
        assert_eq!(legacy_args, "");

        let modern_args = powershell_dev_shell_arch_args(&modern);
        match native_dev_shell_arch() {
            Some("x64") => assert_eq!(modern_args, " -Arch amd64 -HostArch amd64"),
            Some("arm64") => assert_eq!(modern_args, " -Arch arm64 -HostArch amd64"),
            other => assert_eq!(modern_args, "", "on {other:?} the x86 default stands"),
        }

        std::fs::remove_file(modern).unwrap();
        std::fs::remove_file(legacy).unwrap();
        std::fs::remove_dir(temp_dir).unwrap();
    }

    #[test]
    fn test_msys2_profile_from_installation_root() {
        let temp_dir = tempfile::tempdir().unwrap();
        let bash = temp_dir.path().join("usr").join("bin").join("bash.exe");
        std::fs::create_dir_all(bash.parent().unwrap()).unwrap();
        std::fs::write(&bash, b"").unwrap();

        let profile = msys2_profile_from_root(temp_dir.path()).unwrap();

        assert_eq!(profile.id, "msys2");
        assert_eq!(profile.title, "MSYS2");
        assert_eq!(profile.program, bash.to_string_lossy());
        assert_eq!(profile.args, vec!["--login", "-i"]);
        assert!(!profile.is_default);
        assert!(msys2_profile_from_root(&temp_dir.path().join("missing")).is_none());
    }

    #[test]
    fn test_git_bash_profile_from_executable() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path().join("Git");
        let git = root.join("mingw64").join("bin").join("git.exe");
        let bin_bash = root.join("bin").join("bash.exe");
        let usr_bin_bash = root.join("usr").join("bin").join("bash.exe");
        for path in [&git, &bin_bash, &usr_bin_bash] {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, b"").unwrap();
        }

        let profile = git_bash_profile_from_executable(&git).unwrap();
        assert_eq!(profile.id, "git-bash");
        assert_eq!(profile.title, "Git Bash");
        assert_eq!(profile.program, bin_bash.to_string_lossy());
        assert_eq!(profile.args, vec!["--login", "-i"]);
        assert!(!profile.is_default);

        std::fs::remove_file(bin_bash).unwrap();
        let profile = git_bash_profile_from_executable(&git).unwrap();
        assert_eq!(profile.program, usr_bin_bash.to_string_lossy());
        assert!(git_bash_profile_from_executable(&temp_dir.path().join("git.exe")).is_none());
    }

    #[test]
    fn test_append_missing_profiles_preserves_existing_and_default() {
        let mut profiles = vec![bash_profile(
            "windows-terminal-git-bash",
            "Git Bash",
            PathBuf::from(r"C:\Git\bin\bash.exe"),
        )];
        profiles[0].is_default = true;

        append_missing_profiles(
            &mut profiles,
            [
                bash_profile(
                    "git-bash",
                    "git bash",
                    PathBuf::from(r"C:\Git\bin\bash.exe"),
                ),
                bash_profile(
                    "msys2",
                    "MSYS2",
                    PathBuf::from(r"C:\msys64\usr\bin\bash.exe"),
                ),
            ],
        );

        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].id, "windows-terminal-git-bash");
        assert!(profiles[0].is_default);
        assert_eq!(profiles[1].id, "msys2");
        assert!(!profiles[1].is_default);
    }

    #[test]
    fn test_available_profiles_include_scanned_bash_profiles() {
        let scanned = [msys2_profile(), git_bash_profile()];
        let profiles = available_terminal_profiles();

        for profile in scanned.into_iter().flatten() {
            assert!(Path::new(&profile.program).is_file());
            assert!(profiles
                .iter()
                .any(|existing| same_terminal_profile(existing, &profile)));
        }
    }

    #[test]
    fn test_parse_windows_terminal_settings() {
        let profiles = parse_windows_terminal_settings(
            r#"{
                "defaultProfile": "{61c54bbd-c2c6-5271-96e7-009a87ff44bf}",
                "profiles": {
                    "list": [
                        {
                            "guid": "{61c54bbd-c2c6-5271-96e7-009a87ff44bf}",
                            "name": "Windows PowerShell",
                            "commandline": "powershell.exe"
                        },
                        {
                            "guid": "{0caa0dad-35be-5f56-a8ff-afceeeaa6101}",
                            "name": "Command Prompt",
                            "commandline": "cmd.exe"
                        },
                        {
                            "guid": "{2c4de342-38b7-51cf-b940-2309a097f518}",
                            "name": "Ubuntu",
                            "commandline": "wsl.exe -d Ubuntu"
                        }
                    ]
                }
            }"#,
        );

        assert_eq!(profiles.len(), 3);
        assert_eq!(profiles[0].title, "Windows PowerShell");
        assert!(profiles[0].is_default);
        assert_eq!(profiles[1].title, "Command Prompt");
        assert_eq!(
            profiles[2].args,
            vec!["-d".to_string(), "Ubuntu".to_string()]
        );
    }

    #[test]
    fn test_split_windows_commandline() {
        let parts = split_windows_commandline(
            r#""C:\Program Files\PowerShell\7\pwsh.exe" -NoLogo -WorkingDirectory "C:\Users\Test User""#,
        );

        assert_eq!(parts[0], r#"C:\Program Files\PowerShell\7\pwsh.exe"#);
        assert_eq!(parts[1], "-NoLogo");
        assert_eq!(parts[2], "-WorkingDirectory");
        assert_eq!(parts[3], r#"C:\Users\Test User"#);
    }

    #[test]
    fn test_split_windows_launch_commandline_with_unquoted_program_path() {
        let temp_dir = std::env::temp_dir().join("vibe terminal tests");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let program_path = temp_dir.join("fake shell.exe");
        std::fs::write(&program_path, b"").unwrap();

        let commandline = format!(
            "{} -NoLogo -WorkingDirectory \"C:\\Users\\Test User\"",
            program_path.display()
        );
        let (program, args) = split_windows_launch_commandline(&commandline).unwrap();

        assert_eq!(program, program_path.to_string_lossy());
        assert_eq!(args[0], "-NoLogo");
        assert_eq!(args[1], "-WorkingDirectory");
        assert_eq!(args[2], r#"C:\Users\Test User"#);

        std::fs::remove_file(program_path).unwrap();
        std::fs::remove_dir(temp_dir).unwrap();
    }

    #[test]
    fn test_parse_wsl_distributions_from_utf16_output() {
        let utf16: Vec<u8> = "Ubuntu\r\nUbuntu-22.04\r\n"
            .encode_utf16()
            .flat_map(|unit| unit.to_le_bytes())
            .collect();

        let distros = parse_wsl_distributions(&utf16);

        assert_eq!(
            distros,
            vec!["Ubuntu".to_string(), "Ubuntu-22.04".to_string()]
        );
    }

    #[test]
    fn test_source_package_display_name() {
        assert_eq!(
            source_package_display_name("CanonicalGroupLimited.Ubuntu22.04LTS_79rhkp1fndgsc"),
            Some("Ubuntu22.04 LTS".to_string())
        );
    }

    #[test]
    fn test_wsl_default_directory_args() {
        assert_eq!(
            wsl_default_directory_args(),
            vec!["--cd".to_string(), "~".to_string()]
        );
    }

    #[test]
    fn test_parse_windows_terminal_settings_preserves_supported_order() {
        let profiles = parse_windows_terminal_settings(
            r#"{
                "defaultProfile": "{61c54bbd-c2c6-5271-96e7-009a87ff44bf}",
                "profiles": {
                    "list": [
                        {
                            "guid": "{61c54bbd-c2c6-5271-96e7-009a87ff44bf}",
                            "name": "Windows PowerShell",
                            "commandline": "C:\\Program Files\\PowerShell\\7\\pwsh.exe"
                        },
                        {
                            "guid": "{0caa0dad-35be-5f56-a8ff-afceeeaa6101}",
                            "name": "Command Prompt",
                            "commandline": "%SystemRoot%\\System32\\cmd.exe"
                        },
                        {
                            "guid": "{b453ae62-4e3d-5e58-b989-0a998ec441b8}",
                            "name": "Azure Cloud Shell",
                            "source": "Windows.Terminal.Azure"
                        },
                        {
                            "guid": "{4ff56d04-d9cf-57ea-bae2-ad396374e7e3}",
                            "name": "Ubuntu 22.04.5 LTS",
                            "source": "CanonicalGroupLimited.Ubuntu22.04LTS_79rhkp1fndgsc"
                        },
                        {
                            "guid": "{0a5901e5-b6e2-53a2-9fd3-a36c5d16c55e}",
                            "name": "Ubuntu-22.04",
                            "source": "Microsoft.WSL"
                        }
                    ]
                }
            }"#,
        );

        assert_eq!(
            profiles
                .iter()
                .map(|profile| profile.title.clone())
                .collect::<Vec<_>>(),
            vec![
                "Windows PowerShell".to_string(),
                "Command Prompt".to_string(),
                "Ubuntu 22.04.5 LTS".to_string(),
                "Ubuntu-22.04".to_string(),
            ]
        );
        assert!(profiles[0].is_default);
    }
}
