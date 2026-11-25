mod api;
mod server;
mod store;
mod types;
mod validation;
mod wwwroot;

pub use server::ConfigHttpServer;
pub use store::{config_file_exists, log_to_file_enabled, ConfigManager};
// The supervised vterm-voice spawn resolves the persisted (root, relative
// model id) pair through the same discovery the model scan uses.
pub(crate) use api::voice_models::resolve_model_path;
pub(crate) use types::generate_server_secret_key;
pub(crate) use types::is_running_under_wsl;
pub(crate) use types::EnvironmentVariable;
pub use types::{
    decrypt_addon_secret, encrypt_addon_secret, AddonEntryConfig, AddonsConfig, ConfigPatchRequest,
    DdnsConfigPatch, ServerConfigPatch,
};
pub(crate) use types::{default_asr_bind_address, AsrConfig, DEFAULT_ASR_PORT};
pub use types::{default_proxy_enabled, DEFAULT_PROXY_IDLE_TIMEOUT_MINUTES};
pub(crate) use types::{
    is_hashed_server_secret_key, is_protected_server_secret_key, stored_server_secret_key_hash,
    EnvironmentGroup, EnvironmentNode, McpConfig, McpTemplateConfig, RemoteDesktopConfig,
    StaticHttpBindAddress, TerminalEnvVar, TerminalObserverPatternKind, TerminalServerConfig,
    UartFlowControl, UartParity, UartPortConfig, UartStopBits, DEFAULT_CONFIG_PORT,
};

/// Settings that are safe and useful to advertise to desktop/mobile clients.
pub(crate) fn public_server_info(
    config: &TerminalServerConfig,
) -> vterm_core::protocol::ServerInfo {
    let llm_is_valid = |provider: &types::LlmProviderConfig| {
        provider.enabled
            && provider
                .provider
                .as_deref()
                .is_some_and(|provider| !provider.trim().is_empty())
    };
    vterm_core::protocol::ServerInfo {
        config_port: config.server.config_port,
        voice_enabled: config.asr.enabled,
        voice_polish_enabled: llm_is_valid(&config.llm.voice_polish),
        git_commit_message_llm_valid: llm_is_valid(&config.llm.git.provider),
        translation_llm_valid: llm_is_valid(&config.llm.translation.provider),
        remote_desktop_enabled: config.remote_desktop.enabled,
    }
}
pub use types::{DdnsConfig, NotificationChannel, NotificationConfig, NotificationTargetConfig};

#[cfg(target_os = "windows")]
pub(crate) fn process_is_elevated() -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut returned = 0u32;
        let result = GetTokenInformation(
            token,
            TokenElevation,
            (&mut elevation as *mut TOKEN_ELEVATION).cast(),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        );
        CloseHandle(token);
        result != 0 && elevation.TokenIsElevated != 0
    }
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn process_is_elevated() -> bool {
    false
}
// Headless CLI (`--share`) reuses the webui's interface enumeration and DDNS
// domain computation so console output matches what the settings page shows.
pub(crate) use api::ddns::ddns_domain_from_config;
pub(crate) use api::server::detect_network_interfaces;
pub(crate) use api::sync_mcp_token_environment;
#[cfg(test)]
pub(crate) use types::{
    FcmNotificationConfigPatch, NotificationConfigPatch, NotificationTargetConfigPatch,
    TerminalConfigPatch,
};
