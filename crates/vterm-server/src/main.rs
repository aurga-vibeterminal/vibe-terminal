#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

//! Server crate root. The real entry lives in `app.rs` so the `-dev` binary
//! (`main_dev.rs`) can share it via `include!` — `include!` cannot re-inline
//! a crate root that carries inner attributes like the `windows_subsystem`
//! one above. Release Windows builds are GUI-subsystem: no console window
//! flashes when the autostart Run key or the desktop client launches the
//! server; `adopt_parent_console` in `app.rs` restores shell output for the
//! headless CLI commands, and `allocate_hidden_console` gives the daemon a
//! hidden console so console-subsystem descendants never flash one either.
include!("app.rs");
