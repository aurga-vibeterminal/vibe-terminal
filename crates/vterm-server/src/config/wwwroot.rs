//! Embedded wwwroot: static assets compiled into the server binary.
//!
//! The `webui/dist` folder (the Web UI build) is embedded with rust-embed and
//! served from the site root: `/`, `/index.html`, `/index.js`, `/index.css`,
//! `/favicon.svg` — and any file the build adds later (e.g. `/help/*.png`
//! screenshots) comes along automatically, no code change needed.
//!
//! `debug-embed` is enabled so debug builds embed the assets at compile time
//! as well (the old `include_bytes!` behaviour): a missing `webui/dist` is a
//! compile error, and what is served never depends on the checkout.
//! Content types are derived from the file extension, so new asset kinds
//! (images, fonts, media) are served with the right `Content-Type` without
//! touching this file.

use std::borrow::Cow;

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../webui/dist"]
struct WebuiDist;

pub(crate) struct EmbeddedAsset {
    pub(crate) content_type: &'static str,
    pub(crate) cache_control: &'static str,
    pub(crate) bytes: Cow<'static, [u8]>,
}

/// Look up a site path in the embedded assets.
///
/// `/` and `/index.html` resolve to the webui entry document; everything else
/// is resolved relative to the webui root. A path that does not name an
/// embedded file returns `None` (the caller answers 404). Matching is exact
/// against the embedded file list, so traversal-style paths can never escape
/// the embedded folder.
pub(crate) fn get(path: &str) -> Option<EmbeddedAsset> {
    let file_name = match path {
        "/" | "/index.html" => "index.html",
        p if p.starts_with('/') => p.strip_prefix('/')?,
        _ => return None,
    };
    if file_name.is_empty() || file_name.contains(['\\', '\0']) || file_name.contains("..") {
        return None;
    }

    let file = WebuiDist::get(file_name)?;

    Some(EmbeddedAsset {
        content_type: content_type(file_name),
        // The entry document may be revalidated (no-cache); every other asset
        // is served unhashed by Vite's build config, so it must never be
        // stored by the browser.
        cache_control: if path == "/" || path == "/index.html" {
            "no-cache"
        } else {
            "no-cache, no-store, must-revalidate"
        },
        bytes: file.data,
    })
}

/// `Content-Type` for a file extension. Covers the current webui build
/// output plus the common image, font, model and media types so new asset
/// kinds need no code change; unknown extensions fall back to
/// `application/octet-stream`.
fn content_type(file_name: &str) -> &'static str {
    let extension = file_name.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("");
    match extension.to_ascii_lowercase().as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "webmanifest" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        "txt" => "text/plain; charset=utf-8",
        "gltf" => "model/gltf+json",
        "glb" => "model/gltf-binary",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::get;

    #[test]
    fn serves_root_and_index_as_same_asset() {
        let root = get("/").unwrap();
        let index = get("/index.html").unwrap();

        assert_eq!(root.bytes, index.bytes);
        assert_eq!(root.content_type, "text/html; charset=utf-8");
        assert_eq!(root.cache_control, "no-cache");
        assert_eq!(index.cache_control, "no-cache");
    }

    #[test]
    fn serves_script_assets() {
        let asset = get("/index.js").unwrap();

        assert_eq!(asset.content_type, "application/javascript; charset=utf-8");
        assert_eq!(asset.cache_control, "no-cache, no-store, must-revalidate");
        assert!(!asset.bytes.is_empty());
    }

    #[test]
    fn serves_style_assets() {
        let asset = get("/index.css").unwrap();

        assert_eq!(asset.content_type, "text/css; charset=utf-8");
        assert!(!asset.bytes.is_empty());
    }

    #[test]
    fn serves_image_assets() {
        let asset = get("/favicon.svg").unwrap();

        assert_eq!(asset.content_type, "image/svg+xml");
        assert!(!asset.bytes.is_empty());
    }

    #[test]
    fn returns_none_for_unknown_asset() {
        assert!(get("/missing.txt").is_none());
        assert!(get("/models/missing.gltf").is_none());
        assert!(get("/models/").is_none());
        assert!(get("index.html").is_none());
    }

    #[test]
    fn rejects_traversal_paths() {
        assert!(get("/../Cargo.toml").is_none());
        assert!(get("/models/../../Cargo.toml").is_none());
    }

}
