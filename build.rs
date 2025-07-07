fn main() {
    // Check if we're building for macOS
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    // Print cargo metadata
    println!("cargo:rerun-if-changed=build.rs");

    if target_os != "macos" {
        // Only warn during build, don't panic - this allows crates.io to verify the package
        println!("cargo:warning=fs_usage_sys only works on macOS. Current target OS: {target_os}");
        println!("cargo:warning=This crate will not function on non-macOS platforms.");

        // Set a cfg flag that we can use to conditionally compile the code
        println!("cargo:rustc-cfg=unsupported_platform");
    } else {
        // Note about required privileges on macOS
        if std::env::var("PROFILE").unwrap_or_default() == "release" {
            println!("cargo:warning=fs_usage_sys requires sudo/root privileges to run fs_usage");
            println!("cargo:warning=Examples must be run with: sudo cargo run --example <name>");
        }
    }
}
