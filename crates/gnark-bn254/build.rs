use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let gnark_ffi_dir = manifest_dir.join("gnark-ffi");

    // Check if Go is available
    if which::which("go").is_err() {
        panic!("Go compiler not found. Please install Go 1.21 or later.");
    }

    // Build the Go library using Make
    let status = Command::new("make")
        .current_dir(&gnark_ffi_dir)
        .status()
        .expect("Failed to execute make");

    if !status.success() {
        panic!("Failed to build gnark FFI library");
    }

    // Tell cargo to link the library
    println!("cargo:rustc-link-search=native={}", gnark_ffi_dir.display());
    println!("cargo:rustc-link-lib=static=gnark_bn254");

    // Platform-specific linker flags
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    match target_os.as_str() {
        "macos" => {
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=Security");
        }
        "linux" => {
            println!("cargo:rustc-link-lib=dylib=pthread");
            println!("cargo:rustc-link-lib=dylib=dl");
        }
        "windows" => {
            println!("cargo:rustc-link-lib=dylib=ws2_32");
            println!("cargo:rustc-link-lib=dylib=userenv");
        }
        _ => {}
    }

    // Rerun if wrapper changes
    println!("cargo:rerun-if-changed=gnark-ffi/wrapper.go");
    println!("cargo:rerun-if-changed=gnark-ffi/go.mod");
}
