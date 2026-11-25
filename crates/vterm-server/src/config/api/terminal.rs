use serde::{Deserialize, Serialize};

use crate::config::types::{
    TerminalEnvVar, TerminalObserverPatternConfig, TerminalObserverPatternKind,
    TerminalServerConfig,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebTerminalPatternConfig {
    id: String,
    kind: String,
    substrings: Vec<String>,
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebTerminalEnvVar {
    name: String,
    value: String,
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebTerminalConfig {
    pub(super) enabled: bool,
    pub(super) auto_confirm: bool,
    pub(super) analysis_delay_seconds: u16,
    pub(super) confirmation_grace_seconds: u16,
    pub(super) notify_on_completion: bool,
    pub(super) patterns: Vec<WebTerminalPatternConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) llm_provider: Option<String>,
    pub(super) env_vars: Vec<WebTerminalEnvVar>,
}

pub(super) fn terminal_config_to_web(config: &TerminalServerConfig) -> WebTerminalConfig {
    WebTerminalConfig {
        enabled: config.terminal.enabled,
        auto_confirm: config.terminal.auto_confirm,
        analysis_delay_seconds: config.terminal.analysis_delay_seconds,
        confirmation_grace_seconds: config.terminal.confirmation_grace_seconds,
        notify_on_completion: config.terminal.notify_on_completion,
        patterns: config
            .terminal
            .patterns
            .iter()
            .map(|pattern| WebTerminalPatternConfig {
                id: pattern.id.clone(),
                kind: match pattern.kind {
                    TerminalObserverPatternKind::Confirmation => "confirmation".to_string(),
                    TerminalObserverPatternKind::Selection => "selection".to_string(),
                    TerminalObserverPatternKind::Completion => "completion".to_string(),
                    TerminalObserverPatternKind::Ignore => "ignore".to_string(),
                },
                substrings: pattern.substrings.clone(),
                enabled: pattern.enabled,
            })
            .collect(),
        llm_provider: config.terminal.llm_provider.clone(),
        env_vars: config
            .terminal
            .env_vars
            .iter()
            .map(|env_var| WebTerminalEnvVar {
                name: env_var.name.clone(),
                value: env_var.value.clone(),
                enabled: env_var.enabled,
            })
            .collect(),
    }
}

pub(super) fn apply_web_terminal_config(
    config: &mut TerminalServerConfig,
    payload: WebTerminalConfig,
) {
    config.terminal.enabled = payload.enabled;
    config.terminal.auto_confirm = payload.auto_confirm;
    config.terminal.analysis_delay_seconds = payload.analysis_delay_seconds;
    config.terminal.confirmation_grace_seconds = payload.confirmation_grace_seconds;
    config.terminal.notify_on_completion = payload.notify_on_completion;
    config.terminal.patterns = payload
        .patterns
        .into_iter()
        .filter_map(|pattern| {
            let id = pattern.id.trim().to_string();
            let substrings = pattern
                .substrings
                .into_iter()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>();

            if id.is_empty() || substrings.is_empty() {
                return None;
            }

            Some(TerminalObserverPatternConfig {
                id,
                kind: match pattern.kind.trim() {
                    "selection" => TerminalObserverPatternKind::Selection,
                    "completion" => TerminalObserverPatternKind::Completion,
                    "ignore" => TerminalObserverPatternKind::Ignore,
                    _ => TerminalObserverPatternKind::Confirmation,
                },
                substrings,
                enabled: pattern.enabled,
            })
        })
        .collect();
    config.terminal.llm_provider = payload
        .llm_provider
        .filter(|value| !value.trim().is_empty());
    // Rows are persisted verbatim — including empty names: those are the
    // user's in-progress edits, and dropping them here makes the autosave
    // round-trip delete the freshly added row from the webui. Empty/blank
    // names are filtered at apply time (session_manager).
    config.terminal.env_vars = payload
        .env_vars
        .into_iter()
        .map(|env_var| TerminalEnvVar {
            name: env_var.name,
            value: env_var.value,
            enabled: env_var.enabled,
        })
        .collect();
}

#[cfg(test)]
mod tests {
    use super::{
        apply_web_terminal_config, terminal_config_to_web, WebTerminalConfig, WebTerminalEnvVar,
    };
    use crate::config::types::TerminalServerConfig;

    /// A just-added editor row has an empty name; the autosave round-trip
    /// must not drop it or it vanishes from the webui seconds after adding.
    #[test]
    fn env_vars_round_trip_keeps_in_progress_rows() {
        let mut config = TerminalServerConfig::default();
        let payload = WebTerminalConfig {
            enabled: true,
            auto_confirm: false,
            analysis_delay_seconds: 3,
            confirmation_grace_seconds: 30,
            notify_on_completion: true,
            patterns: Vec::new(),
            llm_provider: None,
            env_vars: vec![
                WebTerminalEnvVar {
                    name: "FOO".to_string(),
                    value: "bar".to_string(),
                    enabled: true,
                },
                WebTerminalEnvVar {
                    name: String::new(),
                    value: String::new(),
                    enabled: true,
                },
            ],
        };

        apply_web_terminal_config(&mut config, payload);

        assert_eq!(config.terminal.env_vars.len(), 2);
        assert_eq!(terminal_config_to_web(&config).env_vars.len(), 2);
    }
}
