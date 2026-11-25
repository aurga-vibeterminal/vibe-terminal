use super::types::{
    is_supported_uart_terminal_type, ApnsNotificationConfigPatch, CodeAnalysisLlmConfigPatch,
    ConfigError, ConfigPatchRequest, ConfigUpdateResult, DdnsConfigPatch, DeviceConfigPatch,
    EnvironmentNode, EnvironmentVariable, FcmNotificationConfigPatch, LlmConfigPatch,
    LlmProviderConfigPatch, NotificationConfigPatch, NotificationTargetConfig,
    NotificationTargetConfigPatch, ServerConfigPatch, TerminalConfigPatch,
    TerminalObserverPatternConfig, TerminalObserverPatternConfigPatch, TerminalServerConfig,
    TpnsNotificationConfigPatch, TranslationLlmConfigPatch, UartPortConfig,
};
use super::{is_hashed_server_secret_key, is_protected_server_secret_key};

pub(crate) fn apply_patch(
    current: &TerminalServerConfig,
    patch: ConfigPatchRequest,
) -> Result<ConfigUpdateResult, ConfigError> {
    let mut updated = current.clone();
    let mut restart_required = false;

    if let Some(server) = patch.server {
        restart_required |= apply_server_patch(&mut updated, server);
    }

    if let Some(device) = patch.device {
        apply_device_patch(&mut updated, device);
    }

    if let Some(ddns) = patch.ddns {
        apply_ddns_patch(&mut updated, ddns);
    }

    if let Some(notification) = patch.notification {
        apply_notification_patch(&mut updated, notification);
    }

    if let Some(terminal) = patch.terminal {
        apply_terminal_patch(&mut updated, terminal);
    }

    if let Some(llm) = patch.llm {
        apply_llm_patch(&mut updated, llm);
    }

    validate_config(&updated)?;

    Ok(ConfigUpdateResult {
        config: updated,
        restart_required,
    })
}

pub(crate) fn validate_config(config: &TerminalServerConfig) -> Result<(), ConfigError> {
    if config.server.relay_port == 0 {
        return Err(ConfigError::Validation(
            "server.relay_port must be greater than zero".to_string(),
        ));
    }

    if config.server.config_port == 0 {
        return Err(ConfigError::Validation(
            "server.config_port must be greater than zero".to_string(),
        ));
    }

    if config.server.relay_port == config.server.config_port {
        return Err(ConfigError::Validation(
            "server.relay_port and server.config_port must be different".to_string(),
        ));
    }

    if config.server.proxy_port_start == 0 || config.server.proxy_port_end == 0 {
        return Err(ConfigError::Validation(
            "server proxy ports must be greater than zero".to_string(),
        ));
    }

    if config.server.proxy_port_start > config.server.proxy_port_end {
        return Err(ConfigError::Validation(
            "server.proxy_port_start must be less than or equal to server.proxy_port_end"
                .to_string(),
        ));
    }

    if config.server.static_http_port_start == 0
        || config.server.static_http_port_end == 0
        || config.server.static_http_port_start > config.server.static_http_port_end
    {
        return Err(ConfigError::Validation(
            "server static HTTP port range must be valid".to_string(),
        ));
    }

    if config.server.relay_udp_port_start == 0
        || config.server.relay_udp_port_end == 0
        || config.server.relay_udp_port_start > config.server.relay_udp_port_end
    {
        return Err(ConfigError::Validation(
            "server relay UDP port range must be valid".to_string(),
        ));
    }

    let secret_key = config.server.secret_key.trim();
    if secret_key.is_empty() {
        return Err(ConfigError::Validation(
            "server.secret_key must be set".to_string(),
        ));
    }

    if is_hashed_server_secret_key(secret_key) && secret_key.len() != 71 {
        return Err(ConfigError::Validation(
            "server.secret_key stored hash must be a sha256 value".to_string(),
        ));
    }

    if !is_protected_server_secret_key(secret_key) && !(12..=16).contains(&secret_key.len()) {
        return Err(ConfigError::Validation(
            "server.secret_key must be between 12 and 16 characters".to_string(),
        ));
    }

    let relay_secret_key = config.server.relay_secret_key.trim();
    if is_hashed_server_secret_key(relay_secret_key) && relay_secret_key.len() != 71 {
        return Err(ConfigError::Validation(
            "server.relay_secret_key stored hash must be a sha256 value".to_string(),
        ));
    }

    if !relay_secret_key.is_empty()
        && !is_protected_server_secret_key(relay_secret_key)
        && !(12..=16).contains(&relay_secret_key.len())
    {
        return Err(ConfigError::Validation(
            "server.relay_secret_key must be between 12 and 16 characters when set".to_string(),
        ));
    }

    let relay_address = config.server.relay_address.trim();
    if !relay_address.is_empty()
        && !relay_address.starts_with("ws://")
        && !relay_address.starts_with("wss://")
        && !relay_address.starts_with("http://")
        && !relay_address.starts_with("https://")
    {
        return Err(ConfigError::Validation(
            "server.relay_address must start with ws://, wss://, http://, or https://".to_string(),
        ));
    }

    if !relay_address.is_empty() && relay_secret_key.is_empty() {
        return Err(ConfigError::Validation(
            "server.relay_secret_key must be set when relay_address is configured".to_string(),
        ));
    }

    if config.server.relay_strict_tls {
        if !relay_address.starts_with("wss://") && !relay_address.starts_with("https://") {
            return Err(ConfigError::Validation(
                "server.relay_strict_tls requires a wss:// relay address".to_string(),
            ));
        }
        vterm_common::private_ca::root_certificates_from_pem(config.server.relay_ca_pem.trim())
            .map_err(|error| {
                ConfigError::Validation(format!("server.relay_ca_pem is invalid: {}", error))
            })?;
    }

    if (!config.server.cert_path.trim().is_empty() || !config.server.key_path.trim().is_empty())
        && (config.server.cert_path.trim().is_empty() || config.server.key_path.trim().is_empty())
    {
        return Err(ConfigError::Validation(
            "server.cert_path and server.key_path must both be set when using custom TLS files"
                .to_string(),
        ));
    }

    for san in &config.server.tls_extra_sans {
        vterm_common::private_ca::normalize_san(san).map_err(|error| {
            ConfigError::Validation(format!("server.tls_extra_sans: {}", error))
        })?;
    }

    if !config.device.serial_port.trim().is_empty() && config.device.serial_port.trim().len() < 2 {
        return Err(ConfigError::Validation(
            "device.serial_port must be a valid port name".to_string(),
        ));
    }

    if config.device.serial_baud_rate == 0 {
        return Err(ConfigError::Validation(
            "device.serial_baud_rate must be greater than zero".to_string(),
        ));
    }

    let mut uart_ids = std::collections::HashSet::new();
    for port in &config.uart.ports {
        validate_uart_port(port)?;
        if !uart_ids.insert(port.id.clone()) {
            return Err(ConfigError::Validation(format!(
                "uart.ports contains duplicate id '{}'",
                port.id
            )));
        }
    }

    if config.ddns.enabled
        && config
            .ddns
            .provider
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        return Err(ConfigError::Validation(
            "ddns.provider must be set when ddns.enabled is true".to_string(),
        ));
    }

    if config.ddns.polling_interval_minutes == 0 {
        return Err(ConfigError::Validation(
            "ddns.polling_interval_minutes must be greater than zero".to_string(),
        ));
    }

    if config.notification.queue_capacity == 0 {
        return Err(ConfigError::Validation(
            "notification.queue_capacity must be greater than zero".to_string(),
        ));
    }

    if config.notification.max_retries > 10 {
        return Err(ConfigError::Validation(
            "notification.max_retries must be less than or equal to 10".to_string(),
        ));
    }

    if config.notification.apns.enabled
        && config
            .notification
            .apns
            .topic
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        return Err(ConfigError::Validation(
            "notification.apns.topic must be set when APNs is enabled".to_string(),
        ));
    }

    if config.notification.apns.enabled
        && config
            .notification
            .apns
            .certificate_path
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        return Err(ConfigError::Validation(
            "notification.apns.certificate_path must be set when APNs is enabled".to_string(),
        ));
    }

    if config.notification.fcm.enabled
        && config
            .notification
            .fcm
            .project_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        return Err(ConfigError::Validation(
            "notification.fcm.project_id must be set when FCM is enabled".to_string(),
        ));
    }

    if config.notification.tpns.enabled
        && config
            .notification
            .tpns
            .access_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        return Err(ConfigError::Validation(
            "notification.tpns.access_id must be set when TPNS is enabled".to_string(),
        ));
    }

    if config.notification.tpns.enabled
        && config
            .notification
            .tpns
            .secret_key
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        return Err(ConfigError::Validation(
            "notification.tpns.secret_key must be set when TPNS is enabled".to_string(),
        ));
    }

    for target in &config.notification.targets {
        if target.id.trim().is_empty() {
            return Err(ConfigError::Validation(
                "notification.targets[].id must be set".to_string(),
            ));
        }

        if target.device_token.trim().is_empty() {
            return Err(ConfigError::Validation(format!(
                "notification target '{}' must include a device_token",
                target.id
            )));
        }
    }

    if config.terminal.analysis_delay_seconds == 0 {
        return Err(ConfigError::Validation(
            "terminal.analysis_delay_seconds must be greater than zero".to_string(),
        ));
    }

    if config.terminal.confirmation_grace_seconds == 0 {
        return Err(ConfigError::Validation(
            "terminal.confirmation_grace_seconds must be greater than zero".to_string(),
        ));
    }

    for pattern in &config.terminal.patterns {
        validate_terminal_observer_pattern(pattern)?;
    }

    // The synthetic root is a container only — its own id/name are not validated.
    // Recurse into its children, which are real Group/Leaf nodes.
    let mut env_node_ids = std::collections::HashSet::new();
    for child in &config.environment.root.children {
        validate_environment_node(child, &mut env_node_ids)?;
    }

    Ok(())
}

fn validate_environment_node(
    node: &EnvironmentNode,
    seen_ids: &mut std::collections::HashSet<String>,
) -> Result<(), ConfigError> {
    let id = node.id();
    if id.trim().is_empty() {
        return Err(ConfigError::Validation(
            "environment node id must be set".to_string(),
        ));
    }
    if node.name().trim().is_empty() {
        return Err(ConfigError::Validation(
            "environment node name must be set".to_string(),
        ));
    }
    if !seen_ids.insert(id.to_string()) {
        return Err(ConfigError::Validation(format!(
            "environment nodes contain duplicate id '{}'",
            id
        )));
    }

    match node {
        EnvironmentNode::Leaf { variables, .. } => {
            validate_leaf_variables(id, variables)?;
        }
        EnvironmentNode::Group { children, .. } => {
            for child in children {
                validate_environment_node(child, seen_ids)?;
            }
        }
    }
    Ok(())
}

fn validate_leaf_variables(
    leaf_id: &str,
    variables: &[EnvironmentVariable],
) -> Result<(), ConfigError> {
    let mut var_keys = std::collections::HashSet::new();
    for var in variables {
        let key = var.key.trim();
        if key.is_empty() {
            return Err(ConfigError::Validation(
                "environment leaf variables[].key must be set".to_string(),
            ));
        }
        // Environment variables are case-sensitive on most systems (Unix, macOS),
        // so we compare keys exactly rather than normalizing case.
        if !var_keys.insert(key.to_string()) {
            return Err(ConfigError::Validation(format!(
                "environment leaf '{}' contains duplicate variable key '{}'",
                leaf_id, var.key
            )));
        }
    }
    Ok(())
}

fn validate_uart_port(port: &UartPortConfig) -> Result<(), ConfigError> {
    if port.id.trim().is_empty() {
        return Err(ConfigError::Validation(
            "uart.ports[].id must be set".to_string(),
        ));
    }

    if port.name.trim().is_empty() {
        return Err(ConfigError::Validation(
            "uart.ports[].name must be set".to_string(),
        ));
    }

    if port.port.trim().is_empty() {
        return Err(ConfigError::Validation(
            "uart.ports[].port must be set".to_string(),
        ));
    }

    if port.baud_rate == 0 {
        return Err(ConfigError::Validation(
            "uart.ports[].baud_rate must be greater than zero".to_string(),
        ));
    }

    if !(5..=8).contains(&port.data_bits) {
        return Err(ConfigError::Validation(
            "uart.ports[].data_bits must be between 5 and 8".to_string(),
        ));
    }

    if !is_supported_uart_terminal_type(&port.terminal_type) {
        return Err(ConfigError::Validation(
            "uart.ports[].terminal_type must be one of linux, vt100, vt220, xterm, ansi"
                .to_string(),
        ));
    }

    Ok(())
}

fn validate_terminal_observer_pattern(
    pattern: &TerminalObserverPatternConfig,
) -> Result<(), ConfigError> {
    if pattern.id.trim().is_empty() {
        return Err(ConfigError::Validation(
            "terminal.patterns[].id must be set".to_string(),
        ));
    }

    if pattern.substrings.is_empty()
        || pattern
            .substrings
            .iter()
            .all(|value| value.trim().is_empty())
    {
        return Err(ConfigError::Validation(
            "terminal.patterns[].substrings must include at least one value".to_string(),
        ));
    }

    Ok(())
}

fn apply_llm_patch(config: &mut TerminalServerConfig, patch: LlmConfigPatch) {
    if let Some(terminal) = patch.terminal {
        apply_llm_provider_patch(&mut config.llm.terminal.provider, terminal);
    }
    if let Some(git) = patch.git {
        apply_llm_provider_patch(&mut config.llm.git.provider, git);
    }
    if let Some(git_conflict) = patch.git_conflict {
        apply_llm_provider_patch(&mut config.llm.git_conflict.provider, git_conflict);
    }
    if let Some(chat) = patch.chat {
        apply_llm_provider_patch(&mut config.llm.chat, chat);
    }
    if let Some(voice_polish) = patch.voice_polish {
        apply_llm_provider_patch(&mut config.llm.voice_polish, voice_polish);
    }
    if let Some(code_analysis) = patch.code_analysis {
        apply_code_analysis_patch(&mut config.llm.code_analysis, code_analysis);
    }
    if let Some(translation) = patch.translation {
        apply_translation_patch(&mut config.llm.translation, translation);
    }
}

fn apply_code_analysis_patch(
    config: &mut crate::config::types::CodeAnalysisLlmConfig,
    patch: CodeAnalysisLlmConfigPatch,
) {
    apply_llm_provider_patch(
        &mut config.provider,
        LlmProviderConfigPatch {
            enabled: patch.enabled,
            provider: patch.provider,
            user_config: patch.user_config,
            max_failures: patch.max_failures,
            retry_cooldown_secs: patch.retry_cooldown_secs,
        },
    );
    if let Some(output_language) = patch.output_language {
        config.output_language = output_language;
    }
}

fn apply_translation_patch(
    config: &mut crate::config::types::TranslationLlmConfig,
    patch: TranslationLlmConfigPatch,
) {
    apply_llm_provider_patch(
        &mut config.provider,
        LlmProviderConfigPatch {
            enabled: patch.enabled,
            provider: patch.provider,
            user_config: patch.user_config,
            max_failures: patch.max_failures,
            retry_cooldown_secs: patch.retry_cooldown_secs,
        },
    );
    if let Some(output_language) = patch.output_language {
        config.output_language = output_language;
    }
}

fn apply_llm_provider_patch(
    config: &mut crate::config::types::LlmProviderConfig,
    patch: LlmProviderConfigPatch,
) {
    if let Some(enabled) = patch.enabled {
        config.enabled = enabled;
    }
    if let Some(provider) = patch.provider {
        config.provider = provider
            .map(normalize_optional_string)
            .filter(|v| !v.is_empty());
    }
    if let Some(user_config) = patch.user_config {
        config.user_config = user_config;
    }
    if let Some(max_failures) = patch.max_failures {
        config.max_failures = max_failures.max(3);
    }
    if let Some(retry_cooldown_secs) = patch.retry_cooldown_secs {
        config.retry_cooldown_secs = retry_cooldown_secs.max(10);
    }
}

fn apply_server_patch(config: &mut TerminalServerConfig, patch: ServerConfigPatch) -> bool {
    let mut restart_required = false;

    if let Some(relay_port) = patch.relay_port {
        restart_required |= config.server.relay_port != relay_port;
        config.server.relay_port = relay_port;
    }

    if let Some(config_port) = patch.config_port {
        restart_required |= config.server.config_port != config_port;
        config.server.config_port = config_port;
    }

    if let Some(proxy_port_start) = patch.proxy_port_start {
        restart_required |= config.server.proxy_port_start != proxy_port_start;
        config.server.proxy_port_start = proxy_port_start;
    }

    if let Some(proxy_port_end) = patch.proxy_port_end {
        restart_required |= config.server.proxy_port_end != proxy_port_end;
        config.server.proxy_port_end = proxy_port_end;
    }

    if let Some(port) = patch.static_http_port_start {
        config.server.static_http_port_start = port;
    }
    if let Some(port) = patch.static_http_port_end {
        config.server.static_http_port_end = port;
    }
    if let Some(address) = patch.static_http_bind_address {
        config.server.static_http_bind_address = address;
    }

    if let Some(name) = patch.name {
        config.server.name = normalize_optional_string(name);
    }

    if let Some(relay_address) = patch.relay_address {
        config.server.relay_address = normalize_optional_string(relay_address);
    }

    if let Some(secret_key) = patch.secret_key {
        config.server.secret_key = normalize_optional_string(secret_key);
    }

    if let Some(relay_secret_key) = patch.relay_secret_key {
        config.server.relay_secret_key = normalize_optional_string(relay_secret_key);
    }

    if let Some(relay_strict_tls) = patch.relay_strict_tls {
        config.server.relay_strict_tls = relay_strict_tls;
    }

    if let Some(relay_ca_pem) = patch.relay_ca_pem {
        config.server.relay_ca_pem = normalize_optional_string(relay_ca_pem);
    }

    if let Some(cert_path) = patch.cert_path {
        let cert_path = normalize_optional_string(cert_path);
        restart_required |= config.server.cert_path != cert_path;
        config.server.cert_path = cert_path;
    }

    if let Some(key_path) = patch.key_path {
        let key_path = normalize_optional_string(key_path);
        restart_required |= config.server.key_path != key_path;
        config.server.key_path = key_path;
    }

    if let Some(extra_sans) = patch.tls_extra_sans {
        config.server.tls_extra_sans = extra_sans
            .iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect();
    }

    if let Some(log_to_file) = patch.log_to_file {
        // File logging is wired up once at tracing init, before the config
        // manager loads, so a toggle only takes effect on the next launch.
        restart_required |= config.server.log_to_file != log_to_file;
        config.server.log_to_file = log_to_file;
    }

    restart_required
}

fn apply_device_patch(config: &mut TerminalServerConfig, patch: DeviceConfigPatch) {
    if let Some(serial_baud_rate) = patch.serial_baud_rate {
        config.device.serial_baud_rate = serial_baud_rate;
    }

    if let Some(serial_port) = patch.serial_port {
        config.device.serial_port = normalize_optional_string(serial_port);
    }

}

fn apply_ddns_patch(config: &mut TerminalServerConfig, patch: DdnsConfigPatch) {
    if let Some(enabled) = patch.enabled {
        config.ddns.enabled = enabled;
    }

    if let Some(provider) = patch.provider {
        config.ddns.provider = provider.map(normalize_optional_string);
    }

    if let Some(polling_interval_minutes) = patch.polling_interval_minutes {
        config.ddns.polling_interval_minutes = polling_interval_minutes;
    }

    if let Some(ip_family) = patch.ip_family {
        let valid = ["ipv4", "ipv6", "dual"];
        if valid.contains(&ip_family.as_str()) {
            config.ddns.ip_family = ip_family;
        }
    }

    if let Some(user_config) = patch.user_config {
        config.ddns.user_config = user_config;
    }

    if let Some(network_interface) = patch.network_interface {
        config.ddns.network_interface = network_interface.filter(|v| !v.is_empty());
    }
}

fn apply_notification_patch(config: &mut TerminalServerConfig, patch: NotificationConfigPatch) {
    if let Some(enabled) = patch.enabled {
        config.notification.enabled = enabled;
    }

    if let Some(queue_capacity) = patch.queue_capacity {
        config.notification.queue_capacity = queue_capacity;
    }

    if let Some(max_retries) = patch.max_retries {
        config.notification.max_retries = max_retries;
    }

    if let Some(notify_on_startup) = patch.notify_on_startup {
        config.notification.notify_on_startup = notify_on_startup;
    }

    if let Some(targets) = patch.targets {
        config.notification.targets = targets
            .into_iter()
            .map(merge_notification_target_patch)
            .collect();
    }

    if let Some(apns) = patch.apns {
        apply_apns_patch(config, apns);
    }

    if let Some(fcm) = patch.fcm {
        apply_fcm_patch(config, fcm);
    }

    if let Some(tpns) = patch.tpns {
        apply_tpns_patch(config, tpns);
    }
}

fn apply_terminal_patch(config: &mut TerminalServerConfig, patch: TerminalConfigPatch) {
    if let Some(enabled) = patch.enabled {
        config.terminal.enabled = enabled;
    }

    if let Some(auto_confirm) = patch.auto_confirm {
        config.terminal.auto_confirm = auto_confirm;
    }

    if let Some(analysis_delay_seconds) = patch.analysis_delay_seconds {
        config.terminal.analysis_delay_seconds = analysis_delay_seconds;
    }

    if let Some(confirmation_grace_seconds) = patch.confirmation_grace_seconds {
        config.terminal.confirmation_grace_seconds = confirmation_grace_seconds;
    }

    if let Some(notify_on_completion) = patch.notify_on_completion {
        config.terminal.notify_on_completion = notify_on_completion;
    }

    if let Some(patterns) = patch.patterns {
        config.terminal.patterns = patterns
            .into_iter()
            .map(merge_terminal_observer_pattern_patch)
            .collect();
    }

    if let Some(llm_provider) = patch.llm_provider {
        config.terminal.llm_provider = llm_provider
            .map(normalize_optional_string)
            .filter(|v| !v.is_empty());
    }
}

fn merge_notification_target_patch(
    patch: NotificationTargetConfigPatch,
) -> NotificationTargetConfig {
    NotificationTargetConfig {
        id: patch.id.map(normalize_optional_string).unwrap_or_default(),
        channel: patch.channel.unwrap_or_default(),
        device_token: patch
            .device_token
            .map(normalize_optional_string)
            .unwrap_or_default(),
        topic: patch
            .topic
            .map(|value| value.map(normalize_optional_string))
            .unwrap_or(None),
        enabled: patch.enabled.unwrap_or(true),
    }
}

fn apply_apns_patch(config: &mut TerminalServerConfig, patch: ApnsNotificationConfigPatch) {
    if let Some(enabled) = patch.enabled {
        config.notification.apns.enabled = enabled;
    }

    if let Some(topic) = patch.topic {
        config.notification.apns.topic = topic.map(normalize_optional_string);
    }

    if let Some(certificate_path) = patch.certificate_path {
        config.notification.apns.certificate_path = certificate_path.map(normalize_optional_string);
    }

    if let Some(certificate_password) = patch.certificate_password {
        config.notification.apns.certificate_password =
            certificate_password.map(normalize_optional_string);
    }

    if let Some(environment) = patch.environment {
        config.notification.apns.environment = normalize_optional_string(environment);
    }
}

fn apply_fcm_patch(config: &mut TerminalServerConfig, patch: FcmNotificationConfigPatch) {
    if let Some(enabled) = patch.enabled {
        config.notification.fcm.enabled = enabled;
    }

    if let Some(project_id) = patch.project_id {
        config.notification.fcm.project_id = project_id.map(normalize_optional_string);
    }

    if let Some(service_account_json) = patch.service_account_json {
        config.notification.fcm.service_account_json =
            service_account_json.map(normalize_optional_string);
    }
}

fn apply_tpns_patch(config: &mut TerminalServerConfig, patch: TpnsNotificationConfigPatch) {
    if let Some(enabled) = patch.enabled {
        config.notification.tpns.enabled = enabled;
    }

    if let Some(access_id) = patch.access_id {
        config.notification.tpns.access_id = access_id.map(normalize_optional_string);
    }

    if let Some(secret_key) = patch.secret_key {
        config.notification.tpns.secret_key = secret_key.map(normalize_optional_string);
    }
}

fn merge_terminal_observer_pattern_patch(
    patch: TerminalObserverPatternConfigPatch,
) -> TerminalObserverPatternConfig {
    TerminalObserverPatternConfig {
        id: patch.id.map(normalize_optional_string).unwrap_or_default(),
        kind: patch.kind.unwrap_or_default(),
        substrings: patch
            .substrings
            .unwrap_or_default()
            .into_iter()
            .map(normalize_optional_string)
            .filter(|value| !value.is_empty())
            .collect(),
        enabled: patch.enabled.unwrap_or(true),
    }
}

fn normalize_optional_string(value: String) -> String {
    value.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{ConfigPatchRequest, ServerConfigPatch};

    #[test]
    fn apply_patch_marks_restart_required_for_server_changes() {
        let current = TerminalServerConfig::default();
        let result = apply_patch(
            &current,
            ConfigPatchRequest {
                server: Some(ServerConfigPatch {
                    relay_port: Some(9000),
                    ..ServerConfigPatch::default()
                }),
                ..ConfigPatchRequest::default()
            },
        )
        .unwrap();

        assert!(result.restart_required);
        assert_eq!(result.config.server.relay_port, 9000);
    }

    #[test]
    fn rejects_server_secret_key_outside_allowed_length() {
        let mut config = TerminalServerConfig::default();
        config.server.secret_key = "short-key".to_string();

        let error = validate_config(&config).unwrap_err();

        assert!(error
            .to_string()
            .contains("server.secret_key must be between 12 and 16 characters"));
    }

    #[test]
    fn rejects_missing_relay_secret_key_when_relay_is_configured() {
        let mut config = TerminalServerConfig::default();
        config.server.relay_address = "wss://relay.example.com:7878".to_string();
        config.server.relay_secret_key.clear();

        let error = validate_config(&config).unwrap_err();

        assert!(error
            .to_string()
            .contains("server.relay_secret_key must be set when relay_address is configured"));
    }

    #[test]
    fn rejects_invalid_uart_data_bits() {
        let mut config = TerminalServerConfig::default();
        config
            .uart
            .ports
            .push(crate::config::types::UartPortConfig {
                id: "uart-1".to_string(),
                name: "Console".to_string(),
                port: "COM3".to_string(),
                data_bits: 9,
                ..crate::config::types::UartPortConfig::default()
            });

        let error = validate_config(&config).unwrap_err();

        assert!(error
            .to_string()
            .contains("uart.ports[].data_bits must be between 5 and 8"));
    }

    #[test]
    fn apply_patch_updates_device_without_restart() {
        let current = TerminalServerConfig::default();
        let result = apply_patch(
            &current,
            ConfigPatchRequest {
                device: Some(DeviceConfigPatch {
                    serial_baud_rate: Some(230_400),
                    ..DeviceConfigPatch::default()
                }),
                ..ConfigPatchRequest::default()
            },
        )
        .unwrap();

        assert!(!result.restart_required);
        assert_eq!(result.config.device.serial_baud_rate, 230_400);
    }

    #[test]
    fn validate_rejects_enabled_ddns_without_provider() {
        let mut config = TerminalServerConfig::default();
        config.ddns.enabled = true;
        config.ddns.provider = None;

        let error = validate_config(&config).unwrap_err();
        assert!(error.to_string().contains("ddns.provider"));
    }

    #[test]
    fn validate_rejects_notification_targets_without_device_tokens() {
        let mut config = TerminalServerConfig::default();
        config.notification.enabled = true;
        config.notification.targets.push(NotificationTargetConfig {
            id: "phone".to_string(),
            device_token: String::new(),
            ..NotificationTargetConfig::default()
        });

        let error = validate_config(&config).unwrap_err();
        assert!(error.to_string().contains("device_token"));
    }

    #[test]
    fn validate_allows_environment_variable_keys_differing_only_by_case() {
        use crate::config::types::{
            EnvironmentConfig, EnvironmentGroup, EnvironmentNode, EnvironmentVariable,
        };

        let mut config = TerminalServerConfig::default();
        config.environment = EnvironmentConfig {
            root: EnvironmentGroup {
                id: "root".to_string(),
                name: "Root".to_string(),
                children: vec![EnvironmentNode::Leaf {
                    id: "1".to_string(),
                    name: "Dev".to_string(),
                    variables: vec![
                        EnvironmentVariable {
                            key: "HTTPS_PROXY".to_string(),
                            value: "https://upper.example".to_string(),
                        },
                        EnvironmentVariable {
                            key: "https_proxy".to_string(),
                            value: "https://lower.example".to_string(),
                        },
                    ],
                }],
            },
        };

        validate_config(&config).expect("keys differing only by case should be allowed");
    }

    #[test]
    fn validate_rejects_exact_duplicate_environment_variable_key() {
        use crate::config::types::{
            EnvironmentConfig, EnvironmentGroup, EnvironmentNode, EnvironmentVariable,
        };

        let mut config = TerminalServerConfig::default();
        config.environment = EnvironmentConfig {
            root: EnvironmentGroup {
                id: "root".to_string(),
                name: "Root".to_string(),
                children: vec![EnvironmentNode::Leaf {
                    id: "1".to_string(),
                    name: "Dev".to_string(),
                    variables: vec![
                        EnvironmentVariable {
                            key: "API_KEY".to_string(),
                            value: "first".to_string(),
                        },
                        EnvironmentVariable {
                            key: "API_KEY".to_string(),
                            value: "second".to_string(),
                        },
                    ],
                }],
            },
        };

        let error = validate_config(&config).unwrap_err();
        assert!(error
            .to_string()
            .contains("duplicate variable key 'API_KEY'"));
    }

    #[test]
    fn validate_rejects_duplicate_node_id_across_levels() {
        use crate::config::types::{EnvironmentConfig, EnvironmentGroup, EnvironmentNode};

        let mut config = TerminalServerConfig::default();
        // A group "1" containing a leaf that reuses id "1".
        config.environment = EnvironmentConfig {
            root: EnvironmentGroup {
                id: "root".to_string(),
                name: "Root".to_string(),
                children: vec![EnvironmentNode::Group {
                    id: "1".to_string(),
                    name: "Inner".to_string(),
                    children: vec![EnvironmentNode::Leaf {
                        id: "1".to_string(),
                        name: "Deep".to_string(),
                        variables: Vec::new(),
                    }],
                }],
            },
        };

        let error = validate_config(&config).unwrap_err();
        assert!(error.to_string().contains("duplicate id '1'"));
    }

    #[test]
    fn validate_rejects_group_with_no_invalid_state() {
        // A group with no children and a leaf with no variables are both valid.
        use crate::config::types::{EnvironmentConfig, EnvironmentGroup, EnvironmentNode};

        let mut config = TerminalServerConfig::default();
        config.environment = EnvironmentConfig {
            root: EnvironmentGroup {
                id: "root".to_string(),
                name: "Root".to_string(),
                children: vec![
                    EnvironmentNode::Group {
                        id: "1".to_string(),
                        name: "Empty group".to_string(),
                        children: Vec::new(),
                    },
                    EnvironmentNode::Leaf {
                        id: "2".to_string(),
                        name: "Empty leaf".to_string(),
                        variables: Vec::new(),
                    },
                ],
            },
        };

        validate_config(&config).expect("empty groups and leaves are valid");
    }
}
