use std::path::PathBuf;

use anyhow::{Context, Result};

pub fn codelldb_adapter_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to resolve home directory")?;
    let mut path = home
        .join(".vibeterminal")
        .join("extensions")
        .join("codelldb")
        .join("adapter")
        .join("codelldb");
    if cfg!(target_os = "windows") {
        path.set_extension("exe");
    }
    if !path.exists() {
        anyhow::bail!(
            "CodeLLDB adapter not found at {}. Install codelldb-win32-x64 under ~/.vibeterminal/extensions first.",
            path.display()
        );
    }
    Ok(path)
}
