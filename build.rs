fn main() {
    // Ensure we're building for macOS
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os != "macos" {
        panic!(
            "fs_usage_sys only works on macOS. Current target OS: {}",
            target_os
        );
    }

    // Print cargo metadata
    println!("cargo:rerun-if-changed=build.rs");

    // Note about required privileges
    if std::env::var("PROFILE").unwrap_or_default() == "release" {
        println!("cargo:warning=fs_usage_sys requires sudo/root privileges to run fs_usage");
        println!("cargo:warning=Examples must be run with: sudo cargo run --example <name>");
    }
}

