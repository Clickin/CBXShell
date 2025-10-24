///! Build script for CBXShell

fn main() {
    eprintln!("=== CBXShell Build Script Started ===");

    // Detect if building binary or library
    let target = std::env::var("CARGO_BIN_NAME").ok();
    let crate_name = std::env::var("CARGO_CRATE_NAME").unwrap_or_default();
    let cargo_pkg_name = std::env::var("CARGO_PKG_NAME").unwrap_or_default();

    eprintln!("CARGO_BIN_NAME: {:?}", target);
    eprintln!("CARGO_CRATE_NAME: {}", crate_name);
    eprintln!("CARGO_PKG_NAME: {}", cargo_pkg_name);

    // Only apply WINDOWS subsystem to the DLL library build
    // Binary (cbxmanager) will use default console subsystem
    if target.is_none() && crate_name == "cbxshell" {
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
        eprintln!("Applied WINDOWS subsystem for DLL");
    }

    // Link to Windows libraries for all builds
    println!("cargo:rustc-link-lib=dylib=ole32");
    println!("cargo:rustc-link-lib=dylib=oleaut32");
    println!("cargo:rustc-link-lib=dylib=uuid");
    println!("cargo:rustc-link-lib=dylib=shell32");
    println!("cargo:rustc-link-lib=dylib=gdi32");
    println!("cargo:rustc-link-lib=dylib=user32");
    println!("cargo:rustc-link-lib=dylib=comctl32");

    // Always compile resources - they'll only be linked into binary builds
    eprintln!("=== Compiling Windows resources (icon + manifest) ===");
    eprintln!("Icon path: ../Assets/cbx_icon.ico");
    eprintln!("Manifest path: cbxmanager.exe.manifest");

    let mut res = winres::WindowsResource::new();
    res.set_icon("../Assets/cbx_icon.ico");
    res.set_manifest_file("cbxmanager.exe.manifest");

    match res.compile() {
        Ok(()) => eprintln!("✓ Resources compiled successfully"),
        Err(e) => {
            eprintln!("✗ Failed to compile resources: {}", e);
            // Don't panic - library builds don't need resources
            eprintln!("   (This is OK for library builds)");
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=cbxmanager.exe.manifest");
    println!("cargo:rerun-if-changed=../Assets/cbx_icon.ico");

    eprintln!("=== Build Script Completed ===");
}
