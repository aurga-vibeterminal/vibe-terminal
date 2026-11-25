use vterm_common::SessionAppStatus;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddonManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub email: String,
    pub date: String,
    #[serde(rename = "type")]
    pub addon_type: String,
    #[serde(default)]
    pub config_fields: Vec<AddonConfigField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddonConfigField {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub field_type: AddonFieldType,
    #[serde(default)]
    pub required: bool,
    /// Accepts the JSON key `default` used by the builtin manifests (and
    /// emitted as `default_value` when serialized).
    #[serde(default, alias = "default")]
    pub default_value: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AddonFieldType {
    #[default]
    Text,
    Password,
    Boolean,
    Number,
    Integer,
    Textarea,
}

#[derive(Debug, Clone)]
pub struct AddonInstance {
    pub manifest: AddonManifest,
    pub source: String,
    pub builtin: bool,
    /// True when a user-authored source override (a `<id>.js` file in the
    /// user addons directory) is overriding this addon's original source.
    /// Always `false` for builtins that the user hasn't edited and for
    /// freshly-installed uploaded addons (those are `builtin: false`).
    pub source_overridden: bool,
}

#[derive(Debug, Clone, Default)]
pub struct AddonRunResult {
    pub success: bool,
    pub logs: Vec<String>,
    pub error: Option<String>,
    pub ip: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AppDetectResult {
    pub status: SessionAppStatus,
    pub message: Option<String>,
    /// Clear output-derived Busy after this long without matching output.
    pub idle_after_ms: Option<u64>,
    /// Refresh an existing output heartbeat without starting a new Busy state.
    pub refresh_only: bool,
    /// True when the app has exited the terminal session.
    pub stopped: bool,
}
