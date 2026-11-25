use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use tracing::{debug, info, warn};
use zip::{ZipArchive, ZipWriter};

use super::types::{AddonInstance, AddonManifest};

const CLOUDFLARE_DDNS_MANIFEST: &str = include_str!("builtins/cloudflare-ddns.json");
const CLOUDFLARE_DDNS_SOURCE: &str = include_str!("builtins/cloudflare-ddns.js");

const CLAUDE_CODE_MANIFEST: &str = include_str!("builtins/claude-code.json");
const CLAUDE_CODE_SOURCE: &str = include_str!("builtins/claude-code.js");

const CODEX_MANIFEST: &str = include_str!("builtins/codex.json");
const CODEX_SOURCE: &str = include_str!("builtins/codex.js");

const DEEPSEEK_HARNESS_MANIFEST: &str = include_str!("builtins/deepseek-harness.json");
const DEEPSEEK_HARNESS_SOURCE: &str = include_str!("builtins/deepseek-harness.js");

const KIMI_CODE_MANIFEST: &str = include_str!("builtins/kimi-code.json");
const KIMI_CODE_SOURCE: &str = include_str!("builtins/kimi-code.js");

const QWEN_CODE_MANIFEST: &str = include_str!("builtins/qwen-code.json");
const QWEN_CODE_SOURCE: &str = include_str!("builtins/qwen-code.js");

const PI_MANIFEST: &str = include_str!("builtins/pi.json");
const PI_SOURCE: &str = include_str!("builtins/pi.js");

const OPENCODE_MANIFEST: &str = include_str!("builtins/opencode.json");
const OPENCODE_SOURCE: &str = include_str!("builtins/opencode.js");

const OPENAI_LLM_MANIFEST: &str = include_str!("builtins/openai-llm.json");
const OPENAI_LLM_SOURCE: &str = include_str!("builtins/openai-llm.js");

const VIBE_VOICE_MANIFEST: &str = include_str!("builtins/vibe-voice.json");
const VIBE_VOICE_SOURCE: &str = include_str!("builtins/vibe-voice.js");

const EASYEDA_MANIFEST: &str = include_str!("builtins/easyeda.json");
const EASYEDA_SOURCE: &str = include_str!("builtins/easyeda.js");

/// Addon `type` values the host recognizes. Uploads declaring anything else
/// are rejected. The WebUI further collapses these into display groups
/// (app / llm / voice / dns / mcp), but the backend stores the raw spelling
/// so an addon authored as `asr` vs `voice` round-trips unchanged.
const SUPPORTED_ADDON_TYPES: &[&str] = &["app", "llm", "asr", "stt", "voice", "ddns", "dns", "mcp"];

/// Error surfaced to the API layer. Strings are user-visible (returned as
/// `BadRequest` bodies), so they're kept short and actionable.
pub type AddonHostResult<T> = Result<T, String>;

/// A loaded add-on registry.
///
/// Add-ons come from two places:
/// 1. **Builtins** — compiled into the binary via `include_str!`. These are
///    the pristine source of truth for their manifest and source.
/// 2. **User add-on directory** (`<config_dir>/.vibeterminal/addons/`):
///   - `<id>.js` overrides the source of *any* addon that shares the id
///     (builtin or uploaded). This is the "snapshot" that survives restart.
///   - `<id>.json` + `<id>.js` together register a brand-new uploaded addon.
///     For built-in ids the `.json` is ignored (builtins own their manifest).
///
/// The instance map lives behind a `RwLock` so the API can hot-swap entries
/// (edit / upload / delete) without restarting the server — executors read a
/// cheap clone of the needed `AddonInstance`.
pub struct AddonHost {
    /// Pristine builtins, never mutated. Used both for initial population and
    /// as the restore point when a source override is reset.
    builtins: HashMap<String, AddonInstance>,
    /// Live instances: builtins with any user overrides applied, plus any
    /// uploaded user addons.
    instances: RwLock<HashMap<String, AddonInstance>>,
    /// Directory holding user overrides and uploaded addons. Empty when no
    /// directory could be resolved (e.g. some test contexts); in that case
    /// the host still serves builtins but save/upload are no-ops that error.
    addons_dir: Option<PathBuf>,
}

impl AddonHost {
    pub fn new() -> Self {
        let dir = default_addons_dir();
        let host = Self::with_dir(dir);
        host.reload();
        host
    }

    /// Construct with an explicit addons directory (used by `new` and tests
    /// that want isolation). Does *not* load — callers must call [`reload`].
    fn with_dir(addons_dir: Option<PathBuf>) -> Self {
        info!("AddonHost initializing");
        let mut builtins = HashMap::new();
        register_builtin(
            &mut builtins,
            CLOUDFLARE_DDNS_MANIFEST,
            CLOUDFLARE_DDNS_SOURCE,
            "cloudflare-ddns",
        );
        register_builtin(
            &mut builtins,
            CLAUDE_CODE_MANIFEST,
            CLAUDE_CODE_SOURCE,
            "claude-code",
        );
        register_builtin(&mut builtins, CODEX_MANIFEST, CODEX_SOURCE, "codex");
        register_builtin(
            &mut builtins,
            DEEPSEEK_HARNESS_MANIFEST,
            DEEPSEEK_HARNESS_SOURCE,
            "deepseek-harness",
        );
        register_builtin(
            &mut builtins,
            KIMI_CODE_MANIFEST,
            KIMI_CODE_SOURCE,
            "kimi-code",
        );
        register_builtin(
            &mut builtins,
            QWEN_CODE_MANIFEST,
            QWEN_CODE_SOURCE,
            "qwen-code",
        );
        register_builtin(&mut builtins, PI_MANIFEST, PI_SOURCE, "pi");
        register_builtin(
            &mut builtins,
            OPENCODE_MANIFEST,
            OPENCODE_SOURCE,
            "opencode",
        );
        register_builtin(
            &mut builtins,
            OPENAI_LLM_MANIFEST,
            OPENAI_LLM_SOURCE,
            "openai-llm",
        );
        register_builtin(
            &mut builtins,
            VIBE_VOICE_MANIFEST,
            VIBE_VOICE_SOURCE,
            "vibe-voice",
        );
        register_builtin(&mut builtins, EASYEDA_MANIFEST, EASYEDA_SOURCE, "easyeda");

        let host = Self {
            builtins,
            instances: RwLock::new(HashMap::new()),
            addons_dir,
        };
        info!(
            "AddonHost ready with {} builtin addon(s)",
            host.builtins.len()
        );
        host
    }

    /// Rebuild the live instance map from builtins + the user addons dir.
    /// Safe to call repeatedly (used after every edit/upload/delete).
    pub fn reload(&self) {
        let mut live: HashMap<String, AddonInstance> = self
            .builtins_iter()
            .map(|i| (i.manifest.id.clone(), i.clone()))
            .collect();

        if let Some(dir) = &self.addons_dir {
            apply_user_dir(&mut live, &self.builtins, dir);
        }

        let mut guard = self
            .instances
            .write()
            .expect("addon instances lock poisoned");
        let count = live.len();
        *guard = live;
        debug!("AddonHost reloaded: {count} addon(s) live");
    }

    /// All live addons, cloned. Cheap enough for the config API which only
    /// runs on page load; executors use the targeted [`get_source`]/`get_instance`
    /// lookups instead.
    pub fn instances(&self) -> Vec<AddonInstance> {
        self.instances
            .read()
            .expect("addon instances lock poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn get_instance(&self, id: &str) -> Option<AddonInstance> {
        self.instances
            .read()
            .expect("addon instances lock poisoned")
            .get(id)
            .cloned()
    }

    pub fn get_source(&self, id: &str) -> Option<String> {
        self.instances
            .read()
            .expect("addon instances lock poisoned")
            .get(id)
            .map(|i| i.source.clone())
    }

    /// Look up a config field's manifest default for an addon. Used by
    /// feature executors to apply a sane fallback when the user has not
    /// stored a value for the field yet.
    pub fn config_field_default(&self, id: &str, key: &str) -> Option<String> {
        self.instances
            .read()
            .expect("addon instances lock poisoned")
            .get(id)?
            .manifest
            .config_fields
            .iter()
            .find(|field| field.key == key)
            .and_then(|field| field.default_value.clone())
    }

    pub fn ddns_addons(&self) -> Vec<AddonInstance> {
        self.instances()
            .into_iter()
            .filter(|i| i.manifest.addon_type == "ddns" || i.manifest.addon_type == "dns")
            .collect()
    }

    pub fn app_addons(&self) -> Vec<AddonInstance> {
        self.instances()
            .into_iter()
            .filter(|i| i.manifest.addon_type == "app")
            .collect()
    }

    pub fn llm_addons(&self) -> Vec<AddonInstance> {
        self.instances()
            .into_iter()
            .filter(|i| i.manifest.addon_type == "llm")
            .collect()
    }

    /// ASR / voice transcription addons. Recognizes the `"asr"`, `"stt"`,
    /// and `"voice"` spellings so the WebUI's existing filter keeps working.
    pub fn asr_addons(&self) -> Vec<AddonInstance> {
        self.instances()
            .into_iter()
            .filter(|i| matches!(i.manifest.addon_type.as_str(), "asr" | "stt" | "voice"))
            .collect()
    }

    /// MCP addons — QuickJS scripts declaring a dynamic MCP tool surface,
    /// served by `mcp::services::addon::AddonMcpService`.
    pub fn mcp_addons(&self) -> Vec<AddonInstance> {
        self.instances()
            .into_iter()
            .filter(|i| i.manifest.addon_type == "mcp")
            .collect()
    }

    /// Directory where overrides / uploads are stored, if one is configured.
    pub fn addons_dir(&self) -> Option<&Path> {
        self.addons_dir.as_deref()
    }

    // ---- mutation: source override --------------------------------------

    /// Persist a user-authored source override for `id` and apply it live.
    ///
    /// Works for both builtins (creates `<id>.js`) and uploaded addons
    /// (rewrites their `<id>.js`). The manifest is never touched here —
    /// editing is source-only by design.
    pub fn save_source_override(&self, id: &str, source: &str) -> AddonHostResult<()> {
        let dir = self.require_dir()?;
        let exists = self
            .get_instance(id)
            .ok_or_else(|| format!("addon not found: {id}"))?;

        fs::create_dir_all(dir).map_err(|e| format!("failed to create addons dir: {e}"))?;
        let js_path = dir.join(format!("{id}.js"));
        fs::write(&js_path, source).map_err(|e| format!("failed to write addon source: {e}"))?;

        let mut updated = exists;
        updated.source = source.to_string();
        // Only builtins carry a "modified" flag — an uploaded addon is
        // already fully user-owned, so editing it doesn't flip the flag (the
        // "Uploaded" badge already conveys ownership, and "Reset" doesn't
        // apply to uploaded addons — they use "Delete" instead).
        updated.source_overridden = updated.builtin;

        let mut guard = self
            .instances
            .write()
            .expect("addon instances lock poisoned");
        guard.insert(id.to_string(), updated);
        info!(
            "Addon '{id}' source override saved ({} bytes)",
            source.len()
        );
        Ok(())
    }

    /// Drop a source override and restore the pristine builtin source.
    /// Only meaningful for builtins; uploaded addons have no builtin to
    /// restore to (use [`delete_addon`] instead).
    pub fn reset_source_override(&self, id: &str) -> AddonHostResult<()> {
        let dir = self.require_dir()?;
        let builtin = self
            .builtins
            .get(id)
            .ok_or_else(|| format!("cannot reset non-builtin addon: {id}"))?;

        let js_path = dir.join(format!("{id}.js"));
        if js_path.exists() {
            fs::remove_file(&js_path)
                .map_err(|e| format!("failed to remove addon override: {e}"))?;
        }

        let mut guard = self
            .instances
            .write()
            .expect("addon instances lock poisoned");
        guard.insert(id.to_string(), builtin.clone());
        info!("Addon '{id}' source override reset to builtin");
        Ok(())
    }

    // ---- mutation: upload / delete --------------------------------------

    /// Install an addon packaged as a zip with `<name>.json` + `<name>.js`
    /// at the archive root (matching the builtin layout). The pair must share
    /// a basename; the manifest's `id` is what gets registered. Unsupported
    /// `type` values are rejected with an error so the UI can surface it.
    pub fn install_zip(&self, bytes: &[u8]) -> AddonHostResult<AddonInstance> {
        let dir = self.require_dir()?;
        let (manifest, source) = parse_addon_zip(bytes)?;

        validate_addon_id(&manifest.id)?;
        if !SUPPORTED_ADDON_TYPES
            .iter()
            .any(|t| *t == manifest.addon_type)
        {
            return Err(format!(
                "unsupported addon type '{}'; expected one of: {}",
                manifest.addon_type,
                SUPPORTED_ADDON_TYPES.join(", ")
            ));
        }

        fs::create_dir_all(dir).map_err(|e| format!("failed to create addons dir: {e}"))?;
        let json_path = dir.join(format!("{}.json", manifest.id));
        let js_path = dir.join(format!("{}.js", manifest.id));
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("failed to encode manifest: {e}"))?;
        fs::write(&json_path, manifest_json)
            .map_err(|e| format!("failed to write addon manifest: {e}"))?;
        fs::write(&js_path, &source).map_err(|e| format!("failed to write addon source: {e}"))?;

        let instance = AddonInstance {
            manifest: manifest.clone(),
            source,
            builtin: false,
            source_overridden: false,
        };

        let mut guard = self
            .instances
            .write()
            .expect("addon instances lock poisoned");
        guard.insert(manifest.id.clone(), instance.clone());
        info!(
            "Addon '{}' installed from upload (type={})",
            instance.manifest.id, instance.manifest.addon_type
        );
        Ok(instance)
    }

    /// Export an addon as a zip containing its manifest (`<id>.json`)
    /// and source (`<id>.js`) — the same layout [`Self::install_zip`]
    /// accepts, so an exported addon can be edited offline and re-uploaded
    /// as-is. For builtins the exported source is the currently effective
    /// one (a user override wins over the pristine builtin source).
    pub fn export_addon_zip(&self, id: &str) -> AddonHostResult<Vec<u8>> {
        let instance = self
            .get_instance(id)
            .ok_or_else(|| format!("addon not found: {id}"))?;
        let manifest_json = serde_json::to_string_pretty(&instance.manifest)
            .map_err(|e| format!("failed to encode manifest: {e}"))?;

        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = ZipWriter::new(&mut cursor);
            let options = zip::write::SimpleFileOptions::default();
            for (name, bytes) in [
                (format!("{id}.json"), manifest_json.as_bytes()),
                (format!("{id}.js"), instance.source.as_bytes()),
            ] {
                writer
                    .start_file(name, options)
                    .map_err(|e| format!("failed to add zip entry: {e}"))?;
                writer
                    .write_all(bytes)
                    .map_err(|e| format!("failed to write zip entry: {e}"))?;
            }
            writer
                .finish()
                .map_err(|e| format!("failed to finalize zip: {e}"))?;
        }
        Ok(cursor.into_inner())
    }

    /// Permanently remove an uploaded addon (both its files and registry
    /// entry). Builtins cannot be deleted — only their override reset.
    pub fn delete_addon(&self, id: &str) -> AddonHostResult<()> {
        let dir = self.require_dir()?;
        let exists = self
            .get_instance(id)
            .ok_or_else(|| format!("addon not found: {id}"))?;
        if exists.builtin {
            return Err(format!("cannot delete builtin addon: {id}"));
        }

        for ext in &["json", "js"] {
            let p = dir.join(format!("{id}.{ext}"));
            if p.exists() {
                fs::remove_file(&p).map_err(|e| format!("failed to remove addon file: {e}"))?;
            }
        }

        let mut guard = self
            .instances
            .write()
            .expect("addon instances lock poisoned");
        guard.remove(id);
        info!("Addon '{id}' deleted");
        Ok(())
    }

    fn require_dir(&self) -> AddonHostResult<&Path> {
        self.addons_dir
            .as_deref()
            .ok_or_else(|| "no addons directory configured".to_string())
    }

    fn builtins_iter(&self) -> impl Iterator<Item = &AddonInstance> {
        self.builtins.values()
    }
}

impl Default for AddonHost {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl AddonHost {
    /// Insert an in-memory addon instance without touching the user addons
    /// directory (executor tests use this to register JS fixtures).
    pub(crate) fn insert_test_instance(&self, id: &str, addon_type: &str, source: &str) {
        let manifest = AddonManifest {
            id: id.to_string(),
            name: id.to_string(),
            description: String::new(),
            author: String::new(),
            email: String::new(),
            date: String::new(),
            addon_type: addon_type.to_string(),
            config_fields: Vec::new(),
        };
        self.instances
            .write()
            .expect("addon instances lock poisoned")
            .insert(
                id.to_string(),
                AddonInstance {
                    manifest,
                    source: source.to_string(),
                    builtin: false,
                    source_overridden: false,
                },
            );
    }
}

fn register_builtin(
    into: &mut HashMap<String, AddonInstance>,
    manifest_json: &str,
    source: &str,
    expected_id: &str,
) {
    let manifest: AddonManifest = serde_json::from_str(manifest_json)
        .unwrap_or_else(|e| panic!("{expected_id} manifest should be valid JSON: {e}"));
    info!(
        "Registering builtin addon: id={}, name={}, type={}",
        manifest.id, manifest.name, manifest.addon_type
    );
    debug!(
        "Addon '{}' config fields: {:?}",
        manifest.id,
        manifest
            .config_fields
            .iter()
            .map(|f| format!("{}({:?})", f.key, f.field_type))
            .collect::<Vec<_>>()
    );
    let id = manifest.id.clone();
    into.insert(
        id,
        AddonInstance {
            manifest,
            source: source.to_string(),
            builtin: true,
            source_overridden: false,
        },
    );
}

/// Fold the user addons directory into a live instance map:
/// - `.js` files override source for any matching id (builtin or uploaded).
/// - `.json` + `.js` pairs register a new uploaded addon when the id is not
///   a builtin. For builtin ids the `.json` is intentionally ignored — we
///   never let a user file mutate a builtin manifest.
fn apply_user_dir(
    live: &mut HashMap<String, AddonInstance>,
    builtins: &HashMap<String, AddonInstance>,
    dir: &Path,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return, // dir doesn't exist yet — nothing to apply
    };

    // Collect available json/js stems.
    let mut json_stems: Vec<String> = Vec::new();
    let mut js_stems: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let is_json = path.extension().and_then(|e| e.to_str()) == Some("json");
        let is_js = path.extension().and_then(|e| e.to_str()) == Some("js");
        if !is_json && !is_js {
            continue;
        }
        let Some(stem) = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
        else {
            continue;
        };
        if is_json {
            json_stems.push(stem);
        } else {
            js_stems.push(stem);
        }
    }

    // Register uploaded addons (json + js, non-builtin id).
    for stem in &json_stems {
        if builtins.contains_key(stem) {
            // Never let a user file redefine a builtin manifest.
            continue;
        }
        if !js_stems.iter().any(|s| s == stem) {
            warn!("Skipping uploaded addon '{stem}': manifest present but no matching .js");
            continue;
        }
        let json_path = dir.join(format!("{stem}.json"));
        let js_path = dir.join(format!("{stem}.js"));
        let manifest_str = match fs::read_to_string(&json_path) {
            Ok(s) => s,
            Err(e) => {
                warn!("Skipping uploaded addon '{stem}': cannot read manifest: {e}");
                continue;
            }
        };
        let manifest: AddonManifest = match serde_json::from_str(&manifest_str) {
            Ok(m) => m,
            Err(e) => {
                warn!("Skipping uploaded addon '{stem}': invalid manifest JSON: {e}");
                continue;
            }
        };
        if manifest.id != *stem {
            warn!(
                "Uploaded addon '{}' ignored: manifest id '{}' does not match filename stem",
                stem, manifest.id
            );
            continue;
        }
        let source = match fs::read_to_string(&js_path) {
            Ok(s) => s,
            Err(e) => {
                warn!("Skipping uploaded addon '{stem}': cannot read source: {e}");
                continue;
            }
        };
        live.insert(
            stem.clone(),
            AddonInstance {
                manifest,
                source,
                builtin: false,
                source_overridden: false,
            },
        );
    }

    // Apply source overrides (.js over any id, including uploaded ones).
    for stem in &js_stems {
        let js_path = dir.join(format!("{stem}.js"));
        // An uploaded addon's own .js is its source, not an override.
        if json_stems.iter().any(|s| s == stem) && !builtins.contains_key(stem) {
            continue;
        }
        let Some(instance) = live.get_mut(stem) else {
            // .js with no matching addon and no builtin — orphan; skip.
            continue;
        };
        let source = match fs::read_to_string(&js_path) {
            Ok(s) => s,
            Err(e) => {
                warn!("Skipping source override for '{stem}': {e}");
                continue;
            }
        };
        instance.source = source;
        instance.source_overridden = instance.builtin;
    }
}

/// Extract the `<name>.json` + `<name>.js` pair from a zip archive. Both
/// files must live at the archive root and share a basename.
fn parse_addon_zip(bytes: &[u8]) -> AddonHostResult<(AddonManifest, String)> {
    let mut archive =
        ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("invalid zip archive: {e}"))?;

    let mut json_name: Option<String> = None;
    let mut js_name: Option<String> = None;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|e| format!("failed to read zip entry: {e}"))?;
        let name = entry.name().to_string();
        // Skip directories and anything not at the root (no path separator).
        if name.ends_with('/') || name.contains('/') {
            continue;
        }
        let lower = name.to_ascii_lowercase();
        if lower.ends_with(".json") && json_name.is_none() {
            json_name = Some(name);
        } else if lower.ends_with(".js") && js_name.is_none() {
            js_name = Some(name);
        }
    }

    let json_name = json_name.ok_or_else(|| "zip must contain a .json manifest".to_string())?;
    let js_name = js_name.ok_or_else(|| "zip must contain a .js source file".to_string())?;

    let json_stem = Path::new(&json_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "manifest file has no name".to_string())?;
    let js_stem = Path::new(&js_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "source file has no name".to_string())?;
    if json_stem != js_stem {
        return Err(format!(
            "zip manifest ('{json_name}') and source ('{js_name}') must share the same basename"
        ));
    }

    let mut manifest_bytes = Vec::new();
    archive
        .by_name(&json_name)
        .map_err(|e| format!("failed to read manifest from zip: {e}"))?
        .read_to_end(&mut manifest_bytes)
        .map_err(|e| format!("failed to read manifest bytes: {e}"))?;
    let manifest: AddonManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("manifest is not valid JSON: {e}"))?;

    let mut source_bytes = String::new();
    archive
        .by_name(&js_name)
        .map_err(|e| format!("failed to read source from zip: {e}"))?
        .read_to_string(&mut source_bytes)
        .map_err(|e| format!("failed to read source bytes: {e}"))?;

    Ok((manifest, source_bytes))
}

fn validate_addon_id(id: &str) -> AddonHostResult<()> {
    if id.is_empty() {
        return Err("addon id cannot be empty".to_string());
    }
    if id.len() > 64 {
        return Err("addon id cannot exceed 64 characters".to_string());
    }
    let valid = id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !valid {
        return Err(format!(
            "addon id '{id}' must contain only a-z, 0-9, '-' or '_'"
        ));
    }
    Ok(())
}

/// Resolve the user addons directory, mirroring the config-store convention
/// in `config/store.rs` (`<home>/.vibeterminal[/addons]`). Returns `None`
/// when no home directory can be determined — the host then serves builtins
/// only and edit/upload endpoints report the error.
fn default_addons_dir() -> Option<PathBuf> {
    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"))?;
    let home = PathBuf::from(home);
    if home.as_os_str().is_empty() {
        return None;
    }
    let cfg = if cfg!(DEVELOPMENT) {
        ".vibeterminal-dev"
    } else {
        ".vibeterminal"
    };
    Some(home.join(cfg).join("addons"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn empty_host_with_temp_dir() -> (AddonHost, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("temp dir");
        let host = AddonHost::with_dir(Some(dir.path().to_path_buf()));
        host.reload();
        (host, dir)
    }

    #[test]
    fn builtins_load_on_construction() {
        let host = AddonHost::new();
        let ids: Vec<_> = host
            .builtins_iter()
            .map(|i| i.manifest.id.clone())
            .collect();
        assert!(ids.contains(&"cloudflare-ddns".to_string()));
        assert!(ids.contains(&"openai-llm".to_string()));
        assert!(ids.contains(&"vibe-voice".to_string()));
        assert!(ids.contains(&"opencode".to_string()));
    }

    #[test]
    fn easyeda_builtin_is_an_mcp_addon_with_timeout_default() {
        let host = AddonHost::new();
        let instance = host.get_instance("easyeda").expect("easyeda builtin");
        assert_eq!(instance.manifest.addon_type, "mcp");
        assert!(instance.builtin);
        assert_eq!(
            host.config_field_default("easyeda", "timeout"),
            Some("120".to_string())
        );
        assert!(host.mcp_addons().iter().any(|i| i.manifest.id == "easyeda"));
    }

    #[test]
    fn save_override_marks_builtin_modified_and_serves_new_source() {
        let (host, _dir) = empty_host_with_temp_dir();

        host.save_source_override("codex", "// edited")
            .expect("save override");
        let inst = host.get_instance("codex").expect("codex present");
        assert!(inst.source_overridden, "builtin should be marked modified");
        assert_eq!(inst.source, "// edited");
        assert_eq!(host.get_source("codex").as_deref(), Some("// edited"));
    }

    #[test]
    fn reset_override_restores_builtin_source() {
        let (host, _dir) = empty_host_with_temp_dir();
        let original = host.get_source("codex").expect("builtin source");

        host.save_source_override("codex", "// edited").unwrap();
        host.reset_source_override("codex").unwrap();

        let inst = host.get_instance("codex").expect("codex present");
        assert!(!inst.source_overridden);
        assert_eq!(inst.source, original);
    }

    #[test]
    fn reset_rejects_non_builtin() {
        let (host, _dir) = empty_host_with_temp_dir();
        let err = host.reset_source_override("does-not-exist").unwrap_err();
        assert!(err.contains("non-builtin"));
    }

    #[test]
    fn install_zip_registers_user_addon_and_rejects_bad_type() {
        let (host, _dir) = empty_host_with_temp_dir();

        let zip = build_zip(
            "my-addon",
            r#"{"id":"my-addon","name":"My","description":"d","author":"a","email":"a@b","date":"2025-01-01","type":"llm"}"#,
            "function chat_completion(){}",
        );
        let installed = host.install_zip(&zip).expect("install ok");
        assert_eq!(installed.manifest.id, "my-addon");
        assert!(!installed.builtin);
        assert!(host.get_instance("my-addon").is_some());

        let bad = build_zip(
            "bad",
            r#"{"id":"bad","name":"B","description":"d","author":"a","email":"a@b","date":"2025-01-01","type":"printer"}"#,
            "// x",
        );
        let err = host.install_zip(&bad).unwrap_err();
        assert!(err.contains("unsupported addon type"), "got: {err}");
    }

    #[test]
    fn delete_addon_removes_uploaded_addon_and_refuses_builtin() {
        let (host, _dir) = empty_host_with_temp_dir();
        let zip = build_zip(
            "my-addon",
            r#"{"id":"my-addon","name":"My","description":"d","author":"a","email":"a@b","date":"2025-01-01","type":"app"}"#,
            "// x",
        );
        host.install_zip(&zip).unwrap();
        host.delete_addon("my-addon").expect("delete");
        assert!(host.get_instance("my-addon").is_none());

        let err = host.delete_addon("codex").unwrap_err();
        assert!(err.contains("builtin"));
    }

    #[test]
    fn reload_picks_up_files_written_directly_to_disk() {
        let (host, dir) = empty_host_with_temp_dir();
        let json = dir.path().join("disk-addon.json");
        let js = dir.path().join("disk-addon.js");
        fs::write(
            &json,
            r#"{"id":"disk-addon","name":"D","description":"d","author":"a","email":"a@b","date":"2025-01-01","type":"ddns"}"#,
        )
        .unwrap();
        fs::write(&js, "// from disk").unwrap();

        host.reload();
        let inst = host.get_instance("disk-addon").expect("picked up");
        assert!(!inst.builtin);
        assert_eq!(inst.source, "// from disk");
    }

    #[test]
    fn validate_id_rules() {
        assert!(validate_addon_id("my-addon_1").is_ok());
        assert!(validate_addon_id("").is_err());
        assert!(validate_addon_id("has space").is_err());
        assert!(validate_addon_id("has/slash").is_err());
        assert!(validate_addon_id(&"a".repeat(65)).is_err());
    }

    fn build_zip(stem: &str, manifest: &str, source: &str) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
            let opts: zip::write::SimpleFileOptions = Default::default();
            zip.start_file(format!("{stem}.json"), opts).unwrap();
            zip.write_all(manifest.as_bytes()).unwrap();
            zip.start_file(format!("{stem}.js"), opts).unwrap();
            zip.write_all(source.as_bytes()).unwrap();
            zip.finish().unwrap();
        }
        buf
    }

    /// Exporting an addon yields a zip in exactly the layout `install_zip`
    /// accepts, so the archive re-installs (and round-trips manifest and
    /// source) unchanged.
    #[test]
    fn export_zip_round_trips_through_install() {
        let (host, _dir) = empty_host_with_temp_dir();
        let zip = build_zip(
            "my-addon",
            r#"{"id":"my-addon","name":"My","description":"d","author":"a","email":"a@b","date":"2025-01-01","type":"llm"}"#,
            "function chat_completion(){}",
        );
        host.install_zip(&zip).expect("install ok");

        let exported = host.export_addon_zip("my-addon").expect("export ok");
        let (manifest, source) = parse_addon_zip(&exported).expect("re-parse ok");
        assert_eq!(manifest.id, "my-addon");
        assert_eq!(manifest.addon_type, "llm");
        assert_eq!(source, "function chat_completion(){}");

        // The exported zip installs again as-is (idempotent reinstall).
        host.install_zip(&exported)
            .expect("exported zip re-installs");
    }

    /// Builtins export too — with the effective (overridden) source when a
    /// user snapshot exists, and the pristine one otherwise.
    #[test]
    fn export_builtin_addon_zip_uses_effective_source() {
        let (host, _dir) = empty_host_with_temp_dir();
        let pristine = host
            .get_instance("openai-llm")
            .expect("builtin present")
            .source
            .clone();

        let exported = host.export_addon_zip("openai-llm").expect("export ok");
        let (manifest, source) = parse_addon_zip(&exported).expect("re-parse ok");
        assert_eq!(manifest.id, "openai-llm");
        assert_eq!(source, pristine);

        // A source override is what gets exported afterwards.
        host.save_source_override("openai-llm", "// patched")
            .expect("override saved");
        let (_, source) =
            parse_addon_zip(&host.export_addon_zip("openai-llm").unwrap()).expect("re-parse ok");
        assert_eq!(source, "// patched");
    }
}
