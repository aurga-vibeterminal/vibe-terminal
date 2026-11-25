#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

//! Development binary (`cargo build --features dev`): identical to the
//! release binary. A thin include so the `-dev` target gets its own path —
//! Cargo warns when two `[[bin]]` targets share one source file.
include!("app.rs");
