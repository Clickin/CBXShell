///! Build script for CBXShell

fn main() {
    // Detect if building binary or library
    let target = std::env::var("CARGO_BIN_NAME").ok();
    let crate_name = std::env::var("CARGO_CRATE_NAME").unwrap_or_default();

    // Only apply WINDOWS subsystem to the DLL library build
    // Binary (cbxmanager) will use default console subsystem
    if target.is_none() && crate_name == "cbxshell" {
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
    }

    // Link to Windows libraries for all builds
    println!("cargo:rustc-link-lib=dylib=ole32");
    println!("cargo:rustc-link-lib=dylib=oleaut32");
    println!("cargo:rustc-link-lib=dylib=uuid");
    println!("cargo:rustc-link-lib=dylib=shell32");
    println!("cargo:rustc-link-lib=dylib=gdi32");
    println!("cargo:rustc-link-lib=dylib=user32");
    println!("cargo:rustc-link-lib=dylib=comctl32");

    // Embed manifest for cbxmanager binary (enables Common Controls v6.0)
    if target.as_deref() == Some("cbxmanager") {
        embed_resource::compile("cbxmanager.rc", embed_resource::NONE);
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=cbxmanager.rc");
    println!("cargo:rerun-if-changed=cbxmanager.exe.manifest");
    println!("cargo:rerun-if-changed=../Assets/cbx_icon.ico");
}
