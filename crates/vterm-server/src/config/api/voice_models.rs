//! Local ASR model discovery and repository-file downloads.
//!
//! Downloads deliberately use the public HTTP repository APIs instead of the
//! Hugging Face or ModelScope command-line clients. A `downloading` marker is
//! retained on disk for incomplete downloads, while the in-progress state is
//! owned exclusively by the current server process.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, OnceLock};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum ModelSource {
    Huggingface,
    Modelscope,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DownloadRequest {
    pub directory: String,
    pub source: ModelSource,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ModelAvailability {
    pub id: String,
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download: Option<DownloadProgress>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ModelScan {
    pub models: Vec<ModelAvailability>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DownloadProgress {
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Clone, Copy)]
struct ModelRepo {
    id: &'static str,
    huggingface: &'static str,
    modelscope: Option<&'static str>,
}

const MODELS: &[ModelRepo] = &[
    ModelRepo {
        id: "qwen3-asr-0.6b",
        huggingface: "Qwen/Qwen3-ASR-0.6B",
        modelscope: Some("Qwen/Qwen3-ASR-0.6B"),
    },
    ModelRepo {
        id: "qwen3-asr-0.6b-8bit",
        huggingface: "mlx-community/Qwen3-ASR-0.6B-8bit",
        modelscope: Some("mlx-community/Qwen3-ASR-0.6B-8bit"),
    },
    ModelRepo {
        id: "qwen3-asr-0.6b-4bit",
        huggingface: "mlx-community/Qwen3-ASR-0.6B-4bit",
        modelscope: Some("mlx-community/Qwen3-ASR-0.6B-4bit"),
    },
    ModelRepo {
        id: "qwen3-asr-1.7b",
        huggingface: "Qwen/Qwen3-ASR-1.7B",
        modelscope: Some("Qwen/Qwen3-ASR-1.7B"),
    },
    ModelRepo {
        id: "qwen3-asr-1.7b-8bit",
        huggingface: "mlx-community/Qwen3-ASR-1.7B-8bit",
        modelscope: Some("mlx-community/Qwen3-ASR-1.7B-8bit"),
    },
    ModelRepo {
        id: "qwen3-asr-1.7b-4bit",
        huggingface: "mlx-community/Qwen3-ASR-1.7B-4bit",
        modelscope: Some("mlx-community/Qwen3-ASR-1.7B-4bit"),
    },
    ModelRepo {
        id: "fun-asr-nano-2512",
        huggingface: "FunAudioLLM/Fun-ASR-Nano-2512",
        modelscope: Some("FunAudioLLM/Fun-ASR-Nano-2512"),
    },
    ModelRepo {
        id: "fun-asr-nano-2512-8bit",
        huggingface: "mlx-community/Fun-ASR-Nano-2512-8bit",
        modelscope: Some("mlx-community/Fun-ASR-Nano-2512-8bit"),
    },
    ModelRepo {
        id: "fun-asr-nano-2512-4bit",
        huggingface: "mlx-community/Fun-ASR-Nano-2512-4bit",
        modelscope: Some("mlx-community/Fun-ASR-Nano-2512-4bit"),
    },
    ModelRepo {
        id: "fun-asr-mlt-nano-2512",
        huggingface: "FunAudioLLM/Fun-ASR-MLT-Nano-2512",
        modelscope: Some("FunAudioLLM/Fun-ASR-MLT-Nano-2512"),
    },
    ModelRepo {
        id: "fun-asr-mlt-nano-2512-8bit",
        huggingface: "mlx-community/Fun-ASR-MLT-Nano-2512-8bit",
        modelscope: Some("mlx-community/Fun-ASR-MLT-Nano-2512-8bit"),
    },
    ModelRepo {
        id: "fun-asr-mlt-nano-2512-4bit",
        huggingface: "mlx-community/Fun-ASR-MLT-Nano-2512-4bit",
        modelscope: Some("mlx-community/Fun-ASR-MLT-Nano-2512-4bit"),
    },
    ModelRepo {
        id: "parakeet-tdt-0.6b-v3",
        huggingface: "nvidia/parakeet-tdt-0.6b-v3",
        modelscope: Some("nv-community/parakeet-tdt-0.6b-v3"),
    },
    ModelRepo {
        id: "parakeet-tdt-0.6b-v3-8bit",
        huggingface: "sonic-speech/parakeet-tdt-0.6b-v3-int8",
        modelscope: None,
    },
    ModelRepo {
        id: "parakeet-tdt-0.6b-v3-4bit",
        huggingface: "sonic-speech/parakeet-tdt-0.6b-v3-int4",
        modelscope: None,
    },
];

#[derive(Clone)]
struct ActiveDownload {
    progress: DownloadProgress,
}

struct DownloadManager {
    active: Mutex<HashMap<String, ActiveDownload>>,
}

fn manager() -> &'static Arc<DownloadManager> {
    static MANAGER: OnceLock<Arc<DownloadManager>> = OnceLock::new();
    MANAGER.get_or_init(|| {
        Arc::new(DownloadManager {
            active: Mutex::new(HashMap::new()),
        })
    })
}

/// HTTP client for model downloads, built per call so the proxy config
/// reflects the CURRENT system proxy state (see `vterm_common::proxy`)
/// instead of the process-start env snapshot — a proxy toggled while the
/// server runs must not break (or be skipped by) downloads. The User-Agent
/// is required: ModelScope's signed LFS CDN rejects anonymous HTTP clients
/// (no User-Agent) with 403, even though the redirect URL is otherwise valid.
fn download_client() -> reqwest::Client {
    crate::addon::http::client_builder_for(&vterm_common::proxy::resolve_system_proxy())
        .user_agent(concat!("vibe-terminal/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("ASR model HTTP client")
}

/// Resolve a path whose components may differ in case from the file system.
///
/// Models pulled through ModelScope land in lowercase directories (the
/// internal model id) while repositories cloned from Hugging Face keep their
/// upstream mixed-case names — and Linux file systems are case sensitive, so
/// a stored `model_relative_path` with the wrong case fails even though the
/// directory is right there. Walk the path component by component:
/// components that exist as typed are kept verbatim (no extra syscalls on
/// the happy path), and a missing component is matched against the parent
/// directory's actual entries by lowercased name. Returns `None` as soon as
/// any component cannot be matched.
fn resolve_case_insensitive(path: &Path) -> Option<PathBuf> {
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(name) => {
                let exact = current.join(name);
                if exact.exists() {
                    current = exact;
                    continue;
                }
                let lowered = name.to_string_lossy().to_lowercase();
                let matched = std::fs::read_dir(&current)
                    .ok()?
                    .flatten()
                    .find(|entry| entry.file_name().to_string_lossy().to_lowercase() == lowered)?;
                current = current.join(matched.file_name());
            }
            other => current = current.join(other.as_os_str()),
        }
    }
    Some(current)
}

/// Resolve the persisted (model root, selected-model relative path) pair to
/// the concrete model directory vterm-voice loads.
///
/// `model_relative_path` stores the internal model id, but the on-disk
/// directory name is whatever the downloader or the user chose — a manually
/// cloned repository keeps its upstream name (`Qwen3-ASR-0.6B-4bit`), so the
/// plain join is only a fast path. A join that misses only by case
/// (ModelScope's lowercase id versus a mixed-case directory) is retried
/// case-insensitively before the costlier discovery. When it still holds no
/// weights, ask the same discovery the scan uses where the model actually
/// lives; the join stays the fallback so vterm-voice's "model path does not
/// exist" error still names a sensible directory.
pub(crate) fn resolve_model_path(model_path: &str, model_relative_path: &str) -> String {
    let root = Path::new(model_path.trim());
    let relative = model_relative_path.trim();
    if root.as_os_str().is_empty() || relative.is_empty() {
        return root.display().to_string();
    }
    let joined = root.join(relative);
    if has_model_weights(&joined) {
        return joined.display().to_string();
    }
    if let Some(found) = resolve_case_insensitive(&joined).filter(|path| has_model_weights(path)) {
        return found.display().to_string();
    }
    if let Some(discovered) = discover_models(root).get(relative) {
        return discovered.display().to_string();
    }
    joined.display().to_string()
}

/// Where a model lives (scan) or would be downloaded to (start_download):
/// the content-discovered directory if any, else the same path joined
/// case-insensitively (a directory that differs only by case must be reused,
/// not re-downloaded into a second lowercase copy), else the plain join of
/// the model id.
fn model_destination(
    root: &Path,
    id: &str,
    discovered: &HashMap<&'static str, PathBuf>,
) -> PathBuf {
    discovered
        .get(id)
        .cloned()
        .or_else(|| resolve_case_insensitive(&root.join(id)))
        .unwrap_or_else(|| root.join(id))
}

pub(super) async fn scan(directory: &str) -> ModelScan {
    let root = PathBuf::from(directory.trim());
    let active = manager().active.lock().await.clone();
    // A model directory may have been created by a repository client or by a
    // user, so its name is not reliable.  Identify model roots from their
    // configuration and weight layout instead.  Once a directory has a model
    // configuration, do not walk into it: nested configs belong to that model
    // (for example, FunASR's bundled Qwen directory), not to another model.
    let discovered = discover_models(&root);
    let models = MODELS
        .iter()
        .map(|model| {
            let destination = model_destination(&root, model.id, &discovered);
            let marker = destination.join("downloading");
            let failed_marker = destination.join(".vterm-download-failed");
            let key = download_key(&root, model.id);
            let download = active.get(&key).map(|item| item.progress.clone());
            let available = discovered.contains_key(model.id)
                && !marker.exists()
                && !failed_marker.exists()
                && has_model_weights(&destination);
            ModelAvailability {
                id: model.id.to_string(),
                available,
                path: available.then(|| destination.display().to_string()),
                download,
            }
        })
        .collect();
    ModelScan { models }
}

fn discover_models(root: &Path) -> HashMap<&'static str, PathBuf> {
    let mut models = HashMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let detected = detect_model(&directory);
        if let Some(id) = detected {
            // Keep the closest match when duplicate copies are present.
            models.entry(id).or_insert(directory);
            continue;
        }
        let Ok(entries) = directory.read_dir() else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                pending.push(path);
            }
        }
    }
    models
}

/// Returns an advertised model ID when `directory` has one of our supported
/// ASR configuration signatures.  This deliberately does not use its name.
fn detect_model(directory: &Path) -> Option<&'static str> {
    if let Ok(yaml) = std::fs::read_to_string(directory.join("config.yaml")) {
        if directory.join("model.pt").is_file() {
            return if yaml.contains("use_low_frame_rate: true") {
                Some("fun-asr-nano-2512")
            } else {
                Some("fun-asr-mlt-nano-2512")
            };
        }
    }

    let text = std::fs::read_to_string(directory.join("config.json")).ok()?;
    let config: serde_json::Value = serde_json::from_str(&text).ok()?;
    let model_type = config.get("model_type").and_then(serde_json::Value::as_str);
    let bits = config
        .get("quantization")
        .or_else(|| config.get("quantization_config"))
        .and_then(|value| value.get("bits"))
        .and_then(serde_json::Value::as_u64)
        // The NeMo-MLX Parakeet exports keep their training config intact and
        // store quantization metadata in a separate file.  In particular,
        // their config has `model_type: "tdt"`, not the Transformers
        // `parakeet_tdt` identifier used by the full-precision model.
        .or_else(|| quantization_bits(directory));

    if model_type == Some("funasr") {
        let nano = config
            .pointer("/adaptor/use_low_frame_rate")
            .and_then(serde_json::Value::as_bool)
            .or_else(|| funasr_readme_is_nano(directory))
            .unwrap_or(false);
        return Some(match (nano, bits) {
            (true, Some(8)) => "fun-asr-nano-2512-8bit",
            (true, Some(4)) => "fun-asr-nano-2512-4bit",
            (false, Some(8)) => "fun-asr-mlt-nano-2512-8bit",
            (false, Some(4)) => "fun-asr-mlt-nano-2512-4bit",
            // MLX FunASR exports are quantized; an unknown bit width is not
            // one of the selectable models and must not be misreported.
            (_, _) => return None,
        });
    }
    if model_type == Some("parakeet_tdt")
        || config
            .get("architectures")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| {
                items
                    .iter()
                    .any(|item| item.as_str() == Some("ParakeetForTDT"))
            })
        || is_nemo_mlx_parakeet(&config, directory)
    {
        return Some(match bits {
            Some(8) => "parakeet-tdt-0.6b-v3-8bit",
            Some(4) => "parakeet-tdt-0.6b-v3-4bit",
            _ => "parakeet-tdt-0.6b-v3",
        });
    }

    // Qwen3-ASR configs contain the paired audio/text "thinker" settings.
    // The encoder layer count distinguishes the two supported base sizes.
    let audio = config
        .pointer("/thinker_config/audio_config")
        .or_else(|| config.get("audio_config"));
    let text = config
        .pointer("/thinker_config/text_config")
        .or_else(|| config.get("text_config"));
    let layers = audio
        .and_then(|value| value.get("encoder_layers"))
        .and_then(serde_json::Value::as_u64);
    if text.is_some() {
        return match (layers, bits) {
            (Some(18), Some(8)) => Some("qwen3-asr-0.6b-8bit"),
            (Some(18), Some(4)) => Some("qwen3-asr-0.6b-4bit"),
            (Some(18), _) => Some("qwen3-asr-0.6b"),
            (Some(24), Some(8)) => Some("qwen3-asr-1.7b-8bit"),
            (Some(24), Some(4)) => Some("qwen3-asr-1.7b-4bit"),
            (Some(24), _) => Some("qwen3-asr-1.7b"),
            _ => None,
        };
    }
    None
}

/// Quantized NeMo-MLX Parakeet repositories put this small file alongside the
/// training-style `config.json`, instead of embedding its contents in it.
fn quantization_bits(directory: &Path) -> Option<u64> {
    let text = std::fs::read_to_string(directory.join("quantization_config.json")).ok()?;
    serde_json::from_str::<serde_json::Value>(&text)
        .ok()?
        .get("bits")
        .and_then(serde_json::Value::as_u64)
}

/// The NeMo-MLX export keeps a training-style config whose `model_type: "tdt"`
/// is nested rather than a top-level Transformers field. Require its dedicated
/// quantization file and the characteristic top-level TDT configuration so a
/// different NeMo model is not advertised as this supported Parakeet one.
fn is_nemo_mlx_parakeet(config: &serde_json::Value, directory: &Path) -> bool {
    quantization_bits(directory).is_some_and(|bits| matches!(bits, 4 | 8))
        && config
            .pointer("/model_defaults/tdt_durations")
            .and_then(serde_json::Value::as_array)
            .is_some()
        && directory.join("model.safetensors").is_file()
}

fn funasr_readme_is_nano(directory: &Path) -> Option<bool> {
    let readme = std::fs::read_to_string(directory.join("README.md")).ok()?;
    if readme.contains("Fun-ASR-MLT-Nano") {
        Some(false)
    } else if readme.contains("Fun-ASR-Nano") {
        Some(true)
    } else {
        None
    }
}

fn has_model_weights(directory: &Path) -> bool {
    directory.join("model.pt").is_file()
        || directory.join("model.safetensors").is_file()
        || directory.join("model.safetensors.index.json").is_file()
        || directory.join("pytorch_model.bin").is_file()
        || directory.join("pytorch_model.bin.index.json").is_file()
}

pub(super) async fn start_download(
    id: &str,
    request: DownloadRequest,
) -> Result<ModelAvailability, String> {
    let model = MODELS
        .iter()
        .find(|model| model.id == id)
        .ok_or_else(|| format!("unsupported ASR model '{id}'"))?;
    let root = PathBuf::from(request.directory.trim());
    if request.directory.trim().is_empty() {
        return Err("model directory is required".to_string());
    }
    let repo = match request.source {
        ModelSource::Huggingface => model.huggingface,
        ModelSource::Modelscope => model
            .modelscope
            .ok_or_else(|| format!("ModelScope does not provide '{}'", model.id))?,
    };
    // Do not create a second copy just because a manually downloaded model
    // uses a different directory name (including a different case: see
    // `resolve_case_insensitive`).
    let discovered = discover_models(&root);
    let destination = model_destination(&root, model.id, &discovered);
    let marker = destination.join("downloading");
    let failed_marker = destination.join(".vterm-download-failed");
    let key = download_key(&root, model.id);
    let manager = manager().clone();

    tokio::fs::create_dir_all(&destination)
        .await
        .map_err(|error| error.to_string())?;
    if destination.is_dir()
        && !marker.exists()
        && !failed_marker.exists()
        && destination
            .read_dir()
            .ok()
            .is_some_and(|mut entries| entries.next().is_some())
    {
        return Ok(ModelAvailability {
            id: model.id.to_string(),
            available: true,
            path: Some(destination.display().to_string()),
            download: None,
        });
    }
    let _ = tokio::fs::remove_file(&failed_marker).await;
    let mut active = manager.active.lock().await;
    if let Some(item) = active.get(&key) {
        if item.progress.status != "failed" {
            return Ok(ModelAvailability {
                id: model.id.to_string(),
                available: false,
                path: None,
                download: Some(item.progress.clone()),
            });
        }
        active.remove(&key);
    }
    std::fs::write(&marker, []).map_err(|error| error.to_string())?;
    active.insert(
        key.clone(),
        ActiveDownload {
            progress: DownloadProgress {
                status: "queued",
                progress: Some(0),
                message: None,
            },
        },
    );
    drop(active);

    let repo = repo.to_string();
    let model_id = model.id.to_string();
    tokio::spawn(async move {
        let result = download_repository(&manager, request.source, &repo, &destination, &key).await;
        if let Err(error) = result {
            tracing::warn!(model = %model_id, "ASR model download failed: {error}");
            {
                let mut active = manager.active.lock().await;
                active.insert(
                    key.clone(),
                    ActiveDownload {
                        progress: DownloadProgress {
                            status: "failed",
                            progress: None,
                            message: Some(error),
                        },
                    },
                );
            }
            let _ = tokio::fs::remove_file(destination.join("downloading")).await;
            let _ = tokio::fs::write(destination.join(".vterm-download-failed"), []).await;
        } else {
            let _ = tokio::fs::remove_file(destination.join("downloading")).await;
            let _ = tokio::fs::remove_file(destination.join(".vterm-download-failed")).await;
            manager.active.lock().await.remove(&key);
        }
    });

    Ok(ModelAvailability {
        id: model.id.to_string(),
        available: false,
        path: None,
        download: Some(DownloadProgress {
            status: "queued",
            progress: Some(0),
            message: None,
        }),
    })
}

fn download_key(root: &Path, id: &str) -> String {
    format!("{}::{id}", root.display())
}

#[derive(Clone)]
struct RemoteFile {
    path: String,
    size: u64,
}

async fn download_repository(
    manager: &DownloadManager,
    source: ModelSource,
    repo: &str,
    destination: &Path,
    key: &str,
) -> Result<(), String> {
    let files = match source {
        ModelSource::Huggingface => list_huggingface_files(&download_client(), repo).await?,
        ModelSource::Modelscope => list_modelscope_files(&download_client(), repo).await?,
    };
    let files: Vec<_> = files
        .into_iter()
        .filter(|file| should_download_file(repo, &file.path))
        .collect();
    if files.is_empty() {
        return Err("repository contains no downloadable files".to_string());
    }
    let total = files.iter().map(|file| file.size).sum::<u64>();
    let mut completed = 0u64;
    for file in files {
        let relative = safe_relative_path(&file.path)?;
        let target = destination.join(relative);
        let url = file_url(source, repo, &file.path)?;
        let written = download_file(
            manager, key, &url, &target, file.size, completed, total, &file.path,
        )
        .await?;
        completed = completed.saturating_add(if file.size == 0 { written } else { file.size });
        update_progress(manager, key, completed, total, &file.path).await;
    }
    Ok(())
}

fn should_download_file(repo: &str, path: &str) -> bool {
    !repo.to_ascii_lowercase().ends_with("/parakeet-tdt-0.6b-v3")
        || !path.to_ascii_lowercase().ends_with(".nemo")
}

#[derive(Deserialize)]
struct HuggingFaceFile {
    path: String,
    #[serde(rename = "type")]
    kind: String,
    size: Option<u64>,
}

async fn list_huggingface_files(
    client: &reqwest::Client,
    repo: &str,
) -> Result<Vec<RemoteFile>, String> {
    let url =
        format!("https://huggingface.co/api/models/{repo}/tree/main?recursive=true&expand=false");
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    let files: Vec<HuggingFaceFile> = response.json().await.map_err(|error| error.to_string())?;
    Ok(files
        .into_iter()
        .filter(|file| file.kind == "file")
        .map(|file| RemoteFile {
            path: file.path,
            size: file.size.unwrap_or(0),
        })
        .collect())
}

async fn list_modelscope_files(
    client: &reqwest::Client,
    repo: &str,
) -> Result<Vec<RemoteFile>, String> {
    let url = format!(
        "https://www.modelscope.cn/api/v1/models/{repo}/repo/files?Revision=master&Recursive=true"
    );
    let value: serde_json::Value = client
        .get(url)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?
        .json()
        .await
        .map_err(|error| error.to_string())?;
    let mut files = Vec::new();
    collect_modelscope_files(&value, &mut files);
    files.sort_by(|left, right| left.path.cmp(&right.path));
    files.dedup_by(|left, right| left.path == right.path);
    Ok(files)
}

fn collect_modelscope_files(value: &serde_json::Value, files: &mut Vec<RemoteFile>) {
    match value {
        serde_json::Value::Object(object) => {
            let path = object
                .get("Path")
                .or_else(|| object.get("path"))
                .and_then(|value| value.as_str());
            let kind = object
                .get("Type")
                .or_else(|| object.get("type"))
                .and_then(|value| value.as_str());
            if let (Some(path), Some(kind)) = (path, kind) {
                if kind.eq_ignore_ascii_case("file") || kind.eq_ignore_ascii_case("blob") {
                    let size = object
                        .get("Size")
                        .or_else(|| object.get("size"))
                        .and_then(|value| value.as_u64())
                        .unwrap_or(0);
                    files.push(RemoteFile {
                        path: path.to_string(),
                        size,
                    });
                }
            }
            for child in object.values() {
                collect_modelscope_files(child, files);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                collect_modelscope_files(value, files);
            }
        }
        _ => {}
    }
}

fn file_url(source: ModelSource, repo: &str, path: &str) -> Result<reqwest::Url, String> {
    let mut url = match source {
        ModelSource::Huggingface => reqwest::Url::parse(&format!(
            "https://huggingface.co/{repo}/resolve/main/{path}"
        )),
        ModelSource::Modelscope => reqwest::Url::parse(&format!(
            "https://www.modelscope.cn/api/v1/models/{repo}/repo"
        )),
    }
    .map_err(|error| error.to_string())?;
    match source {
        ModelSource::Huggingface => {
            url.query_pairs_mut().append_pair("download", "true");
        }
        ModelSource::Modelscope => {
            url.query_pairs_mut()
                .append_pair("Revision", "master")
                .append_pair("FilePath", path);
        }
    }
    Ok(url)
}

fn safe_relative_path(path: &str) -> Result<&Path, String> {
    let path = Path::new(path);
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!(
            "repository returned unsafe file path '{}'",
            path.display()
        ));
    }
    Ok(path)
}

async fn update_progress(
    manager: &DownloadManager,
    key: &str,
    completed: u64,
    total: u64,
    file: &str,
) {
    let progress = (total > 0).then(|| ((completed.saturating_mul(100) / total).min(100)) as u8);
    if let Some(item) = manager.active.lock().await.get_mut(key) {
        item.progress = DownloadProgress {
            status: "downloading",
            progress,
            message: Some(file.to_string()),
        };
    }
}

async fn download_file(
    manager: &DownloadManager,
    key: &str,
    url: &reqwest::Url,
    target: &Path,
    expected_size: u64,
    completed: u64,
    total: u64,
    remote_path: &str,
) -> Result<u64, String> {
    if let Some(parent) = target.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    let temporary = target.with_extension(format!(
        "{}part",
        target
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
    ));
    let target_size = tokio::fs::metadata(target)
        .await
        .ok()
        .map(|metadata| metadata.len());
    if expected_size > 0 && target_size == Some(expected_size) {
        return Ok(expected_size);
    }
    if target_size.is_some() {
        tokio::fs::remove_file(target)
            .await
            .map_err(|error| error.to_string())?;
    }

    let mut resume_at = tokio::fs::metadata(&temporary)
        .await
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    if expected_size > 0 && resume_at > expected_size {
        tokio::fs::remove_file(&temporary)
            .await
            .map_err(|error| error.to_string())?;
        resume_at = 0;
    }
    if expected_size > 0 && resume_at == expected_size {
        tokio::fs::rename(&temporary, target)
            .await
            .map_err(|error| error.to_string())?;
        return Ok(expected_size);
    }

    let mut request = download_client().get(url.clone());
    if resume_at > 0 {
        request = request.header(reqwest::header::RANGE, format!("bytes={resume_at}-"));
    }
    let response = request
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    let append = resume_at > 0 && response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
    if !append {
        resume_at = 0;
    }
    let mut stream = response.bytes_stream();
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .truncate(!append)
        .open(&temporary)
        .await
        .map_err(|error| error.to_string())?;
    let mut written = resume_at;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| error.to_string())?;
        file.write_all(&chunk)
            .await
            .map_err(|error| error.to_string())?;
        written = written.saturating_add(chunk.len() as u64);
        update_progress(
            manager,
            key,
            completed.saturating_add(written),
            total,
            remote_path,
        )
        .await;
    }
    file.flush().await.map_err(|error| error.to_string())?;
    tokio::fs::rename(temporary, target)
        .await
        .map_err(|error| error.to_string())?;
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "vterm-voice-models-{tag}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn write_weights(directory: &Path) {
        std::fs::create_dir_all(directory).unwrap();
        std::fs::write(directory.join("model.safetensors"), b"weights").unwrap();
    }

    #[test]
    fn resolve_case_insensitive_matches_components_by_lowercase() {
        let root = temp_root("case");
        let model = root.join("Qwen3-ASR-0.6B").join("qwen3-asr-0.6b-4bit");
        write_weights(&model);

        // Wrong case on every component still resolves to the real path.
        let found =
            resolve_case_insensitive(&root.join("QWEN3-ASR-0.6B").join("Qwen3-ASR-0.6B-4bit"))
                .unwrap();
        assert_eq!(found, model);

        // A component that matches nothing returns None.
        assert_eq!(
            resolve_case_insensitive(&root.join("Qwen3-ASR-0.6B").join("missing")),
            None
        );
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn resolve_model_path_matches_directory_differing_only_by_case() {
        // ModelScope downloads land in the lowercase model id, while a
        // manually cloned HF repository keeps its mixed-case name; the stored
        // lowercase id must still resolve on case-sensitive file systems.
        // No config.json is written, so content discovery cannot find this
        // directory — only the case-insensitive match can.
        let root = temp_root("resolve-case");
        let model = root.join("Qwen3-ASR-0.6B-4bit");
        write_weights(&model);

        assert_eq!(
            resolve_model_path(root.to_str().unwrap(), "qwen3-asr-0.6b-4bit"),
            model.display().to_string(),
        );
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn resolve_model_path_matches_nested_modelscope_directory() {
        // A ModelScope folder nested inside a full-precision clone that
        // discovery itself detects (and therefore does not walk into) is
        // reachable through case-insensitive resolution of the stored
        // relative path.
        let root = temp_root("resolve-nested");
        let parent = root.join("Qwen3-ASR-0.6B");
        std::fs::create_dir_all(&parent).unwrap();
        std::fs::write(
            parent.join("config.json"),
            r#"{
                "model_type": "qwen3_asr",
                "thinker_config": {
                    "audio_config": { "encoder_layers": 18 },
                    "text_config": {}
                }
            }"#,
        )
        .unwrap();
        std::fs::write(parent.join("model.safetensors"), b"weights").unwrap();

        let model = parent.join("qwen3-asr-0.6b-4bit");
        write_weights(&model);

        assert_eq!(
            resolve_model_path(root.to_str().unwrap(), "Qwen3-ASR-0.6B/QWEN3-ASR-0.6B-4BIT"),
            model.display().to_string(),
        );
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn model_destination_reuses_directory_differing_only_by_case() {
        // Without the case-insensitive fallback, start_download would create
        // a duplicate lowercase copy next to the existing mixed-case one.
        let root = temp_root("destination");
        let model = root.join("Parakeet-TDT-0.6B-v3");
        write_weights(&model);

        let discovered = HashMap::new();
        assert_eq!(
            model_destination(&root, "parakeet-tdt-0.6b-v3", &discovered),
            model
        );
        std::fs::remove_dir_all(&root).unwrap();
    }
}
