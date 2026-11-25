fn main() {
    vterm_build::emit_commit_hash();
    if std::env::var_os("CARGO_FEATURE_DEV").is_some() {
        // `cargo build --features dev` → DEVELOPMENT cfg for this crate
        // (dev data dir/ports, -dev binary name). Feature-driven so no
        // .cargo/config.toml rustflags rewriting is needed.
        println!("cargo:rustc-cfg=DEVELOPMENT");
    }
    vterm_build::embed_windows_version_info("AURGA Vibe Terminal Server");
}
