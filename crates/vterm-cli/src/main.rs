mod cli;
mod client;
mod env;
mod error;
mod servers;
mod session;
mod share_payload;
mod terminal;
mod ui;

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashMap;
use tracing_subscriber::EnvFilter;

use crate::cli::Args;
use crate::client::VTermClient;
use crate::env::Environment;
use crate::error::{exit_code, VTermError};
use crate::terminal::Terminal;

/// Resolves the selected profile's owning server so remote profiles launch on the correct target.
fn selected_profile_server_id(
    profiles: &[vterm_common::TerminalProfile],
    profile_id: Option<&String>,
) -> Option<String> {
    profile_id.and_then(|selected_id| {
        profiles
            .iter()
            .find(|profile| profile.id == *selected_id)
            .and_then(|profile| profile.server_id.clone())
    })
}

#[tokio::main]
async fn main() {
    // Initialize logging
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("vtermc=info"));
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();

    match run().await {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(e) => {
            // The interactive pickers leave the terminal cleared; make sure
            // errors are visible on a fresh line.
            if e.downcast_ref::<VTermError>().is_some() || e.to_string().contains("cancelled") {
                eprintln!();
            }
            // Check if it's a VTermError
            if let Some(vterm_err) = e.downcast_ref::<VTermError>() {
                eprintln!("{}", vterm_err);
                std::process::exit(vterm_err.exit_code());
            }

            // Generic error — `{:#}` keeps the full context chain so the
            // root cause (e.g. a config parse failure behind "Failed to
            // save the server") stays visible.
            eprintln!("Error: {e:#}");
            std::process::exit(exit_code::GENERAL_ERROR);
        }
    }
}

async fn run() -> Result<i32> {
    // Parse CLI arguments
    let args = Args::parse();

    // Add a server to the client config (`vtermc -a [share]`) — a
    // management mode that exits after saving.
    if args.add.is_some() {
        return run_add(args.add.as_ref().unwrap().as_deref());
    }

    // Interactive server + terminal selection (`vtermc -s`)
    if args.select {
        return run_select(&args).await;
    }

    // Detect environment
    let env = detect_env(&args)?;

    // Connect to server
    tracing::info!("Connecting to {}", args.server);
    let mut client =
        VTermClient::connect(&args.server)
            .await
            .map_err(|e| VTermError::ConnectionFailed {
                url: args.server.clone(),
                cause: e.to_string(),
            })?;

    // Perform handshake
    client.handshake().await.map_err(|e| {
        if e.to_string().contains("authentication") {
            VTermError::AuthenticationFailed {
                message: e.to_string(),
            }
        } else {
            VTermError::ConnectionFailed {
                url: args.server.clone(),
                cause: e.to_string(),
            }
        }
    })?;

    tracing::info!("Connected to server");

    create_and_stream(&args, &mut client, &env).await
}

/// Detect the local shell/cwd/environment and apply the CLI overrides.
fn detect_env(args: &Args) -> Result<Environment> {
    let mut env = Environment::detect().context("Failed to detect environment")?;

    // Apply working directory override
    if let Some(cwd) = &args.working_dir {
        env.set_cwd(cwd.clone());
    }

    // Inherit environment variables
    if !args.no_env {
        let patterns = args.env_patterns();
        env.inherit_vars(&patterns);
    }

    // Add extra environment variables
    let extra_env = args.parse_extra_env()?;
    env.add_vars(&extra_env);

    tracing::info!("Detected shell: {}", env.shell);
    tracing::info!("Working directory: {}", env.cwd);
    tracing::info!("Environment variables: {}", env.vars.len());
    Ok(env)
}

/// Create a new terminal session on `client` and stream it interactively
/// (or drain piped input when stdin is not a TTY).
async fn create_and_stream(
    args: &Args,
    client: &mut VTermClient,
    env: &Environment,
) -> Result<i32> {
    let profiles = client.list_profiles().await?;

    // Determine profile ID
    let profile_id = match &args.profile {
        Some(Some(id)) => {
            // Profile ID specified
            Some(id.clone())
        }
        Some(None) => {
            // --profile without value - list and select interactively
            if profiles.is_empty() {
                return Err(VTermError::SessionCreationFailed {
                    message: "No terminal profiles available on server".to_string(),
                }
                .into());
            }
            let selected = ui::profile_select::select_profile(&profiles)?;
            Some(selected)
        }
        None => {
            // No profile specified - use shell detection
            env.shell.profile_id()
        }
    };
    let profile_server_id = selected_profile_server_id(&profiles, profile_id.as_ref());

    if let Some(ref id) = profile_id {
        tracing::info!("Using profile: {}", id);
    } else {
        tracing::info!("Using default profile");
    }

    // Get terminal size
    let size = Terminal::size()?;
    tracing::debug!("Terminal size: {}x{}", size.cols, size.rows);

    // Create session
    let env_map = if args.no_env {
        HashMap::new()
    } else {
        env.to_hash_map()
    };

    let session_id = client
        .create_session(
            profile_id,
            profile_server_id,
            size,
            env_map,
            env.cwd.clone(),
            None,
        )
        .await
        .map_err(|e| VTermError::SessionCreationFailed {
            message: e.to_string(),
        })?;

    tracing::info!("Created session: {}", session_id);

    // Terminal messages are only allowed on Terminal-kind connections;
    // upgrade before starting the stream.
    client.upgrade_to_terminal(session_id).await?;

    // Check if we're running in a TTY
    let is_tty = Terminal::is_stdin_tty();

    if is_tty {
        // Interactive mode with raw terminal
        let exit_code = session::run_interactive_session(client, session_id).await?;
        Ok(exit_code)
    } else {
        // Non-interactive mode (piped input)
        let exit_code = session::run_session(client, session_id).await?;
        Ok(exit_code)
    }
}

/// `vtermc -a [share]`: add a server to the client config (`client.json`).
/// With a `vtshare1:` share string argument the name / address / secret
/// key are decrypted from it; without one they are prompted for, with the
/// address validated as `wss://host:port`.
fn run_add(share: Option<&str>) -> Result<i32> {
    let (name, url, secret_key) = match share {
        Some(text) => {
            let shared = share_payload::parse_share_string(text)
                .ok_or_else(|| anyhow::anyhow!("Not a valid vtshare1: share string"))?;
            (shared.name, shared.host, shared.secret_key)
        }
        None => prompt_server()?,
    };

    let server =
        servers::add_server(&name, &url, &secret_key).context("Failed to save the server")?;
    println!("Added server '{}' ({})", server.name, server.url);
    Ok(0)
}

/// Prompt for the server name, address and secret key. The address must be
/// `wss://host:port` (re-prompted until valid); an empty address cancels.
/// An empty name falls back to the address, matching how the server list
/// renders nameless entries. The secret key is masked with `*`.
fn prompt_server() -> Result<(String, String, String)> {
    let name = prompt_line("Server name")?;
    let url = loop {
        let input = prompt_line("Server address (wss://host:port, empty to cancel)")?;
        if input.is_empty() {
            return Err(anyhow::anyhow!("cancelled").into());
        }
        match servers::validate_server_url(&input) {
            Ok(url) => break url,
            Err(e) => eprintln!("{e}"),
        }
    };
    let secret_key = prompt_secret("Secret key")?;
    let name = if name.is_empty() { url.clone() } else { name };
    Ok((name, url, secret_key))
}

/// Read one trimmed line from stdin after printing `prompt`.
fn prompt_line(prompt: &str) -> Result<String> {
    use std::io::{BufRead, Write};

    print!("{prompt}: ");
    std::io::stdout().flush()?;
    let mut line = String::new();
    let read = std::io::stdin().lock().read_line(&mut line)?;
    if read == 0 {
        // EOF (e.g. piped stdin) — treat as cancel.
        println!();
        return Err(anyhow::anyhow!("cancelled").into());
    }
    Ok(line.trim().to_string())
}

/// Prompt for a secret (e.g. the handshake key) without echoing it: each
/// typed character shows as `*`, backspace erases, Ctrl+C cancels (raw mode
/// means no SIGINT). Paste is accepted too. Non-TTY stdin falls back to a
/// plain line read (piped input is not echoed anyway). Returns the trimmed
/// secret.
fn prompt_secret(prompt: &str) -> Result<String> {
    use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use std::io::Write;

    if !Terminal::is_stdin_tty() {
        return prompt_line(prompt);
    }

    print!("{prompt}: ");
    std::io::stdout().flush()?;

    Terminal::enable_raw_mode()?;
    // The reading loop runs in its own closure so an early `?` return
    // (read failure) cannot skip the terminal restore below.
    let result = (|| -> Result<String> {
        let mut secret = String::new();
        loop {
            match event::read().context("Failed to read key press")? {
                Event::Paste(text) => {
                    for ch in text.chars().filter(|c| !c.is_control()) {
                        secret.push(ch);
                        print!("*");
                    }
                    let _ = std::io::stdout().flush();
                }
                Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind,
                    ..
                }) => {
                    // Windows reports Release/Repeat events too — only react
                    // to actual presses, else every key doubles.
                    if kind != KeyEventKind::Press {
                        continue;
                    }
                    match code {
                        KeyCode::Enter => break Ok(secret.trim().to_string()),
                        KeyCode::Backspace if !secret.is_empty() => {
                            secret.pop();
                            print!("\u{8} \u{8}");
                            let _ = std::io::stdout().flush();
                        }
                        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                            break Err(anyhow::anyhow!("cancelled").into());
                        }
                        KeyCode::Char(ch) if !modifiers.contains(KeyModifiers::CONTROL) => {
                            secret.push(ch);
                            print!("*");
                            let _ = std::io::stdout().flush();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    })();
    // Always restore the terminal, even on cancel/error paths.
    let _ = Terminal::disable_raw_mode();
    print!("\r\n");
    let _ = std::io::stdout().flush();
    result
}

/// `vtermc -s`: pick a server from the client config (Local always first),
/// connect, then pick one of that server's existing terminals to attach to
/// (or create a new one). When the server has no terminals at all, a new
/// terminal is created directly without showing a list.
async fn run_select(args: &Args) -> Result<i32> {
    if !Terminal::is_stdin_tty() {
        return Err(anyhow::anyhow!("vtermc -s requires an interactive terminal").into());
    }

    // Server list: Local plus the managed servers from client.json.
    // Pressing `d` on a managed server (then confirming) removes it from
    // client.json and re-shows the list; Local is never deletable.
    let mut servers = servers::load_servers().context("Failed to load the server list")?;
    if servers.is_empty() {
        return Err(anyhow::anyhow!("No servers configured").into());
    }
    let server = loop {
        let labels: Vec<String> = servers.iter().map(|s| s.label()).collect();
        let deletable: Vec<bool> = servers.iter().map(|s| !s.local).collect();
        match ui::select::select_index_deletable("Select a server", &labels, &deletable)? {
            ui::select::Selection::Pick(idx) => {
                break servers.into_iter().nth(idx).unwrap();
            }
            ui::select::Selection::Delete(idx) => {
                let victim = &servers[idx];
                tracing::info!("Deleting server {} ({})", victim.name, victim.url);
                let removed =
                    servers::remove_server(&victim.url).context("Failed to delete the server")?;
                if removed == 0 {
                    tracing::warn!("No matching entry for {} in client.json", victim.url);
                }
                servers.remove(idx);
            }
        }
    };

    tracing::info!("Connecting to {} ({})", server.name, server.url);
    let mut client =
        VTermClient::connect(&server.url)
            .await
            .map_err(|e| VTermError::ConnectionFailed {
                url: server.url.clone(),
                cause: e.to_string(),
            })?;
    client.set_secret_key(if server.secret_key.is_empty() {
        None
    } else {
        Some(server.secret_key.clone())
    });
    client.handshake().await.map_err(|e| {
        if e.to_string().contains("authentication") {
            VTermError::AuthenticationFailed {
                message: e.to_string(),
            }
        } else {
            VTermError::ConnectionFailed {
                url: server.url.clone(),
                cause: e.to_string(),
            }
        }
    })?;
    tracing::info!("Connected to {}", server.name);

    // The server pushes the session list to a fresh primary connection.
    let sessions = client.wait_for_sessions().await?;

    // "New Terminal" is always the first entry; with no existing terminals
    // it is the only choice, so skip the picker entirely.
    let attach_session_id = if sessions.is_empty() {
        None
    } else {
        let mut lines = vec!["New Terminal".to_string()];
        lines.extend(sessions.iter().map(|s| {
            let cwd = s
                .current_working_path
                .as_deref()
                .filter(|p| !p.is_empty())
                .or_else(|| s.viewing_path.as_deref().filter(|p| !p.is_empty()))
                .unwrap_or("-");
            let title = s
                .session_name
                .as_deref()
                .filter(|n| !n.is_empty())
                .unwrap_or(&s.title);
            let server_col = s
                .server_name
                .as_deref()
                .filter(|n| !n.is_empty())
                .unwrap_or("Local");
            format!("{}\t{}\t{}", cwd, title, server_col)
        }));

        let idx = ui::select::select_index("Select a terminal  (cwd\ttitle\tserver)", &lines)?;
        if idx == 0 {
            None
        } else {
            Some(sessions[idx - 1].id)
        }
    };

    if let Some(session_id) = attach_session_id {
        tracing::info!("Attaching to session {}", session_id);

        // Terminal messages are only allowed on Terminal-kind connections.
        client.upgrade_to_terminal(session_id).await?;

        let exit_code = session::run_interactive_session(&mut client, session_id).await?;
        Ok(exit_code)
    } else {
        create_and_stream(args, &mut client, &detect_env(args)?).await
    }
}
