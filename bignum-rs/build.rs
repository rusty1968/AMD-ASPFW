use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_dir = target_dir()
        .join("include")
        .join(&package_name);

    // Create output directory for headers
    std::fs::create_dir_all(&output_dir).expect("Couldn't create output directory");

    // Generate C header using cbindgen
    generate_c_header(&crate_dir, &output_dir);

    // Copy static header file
    copy_static_header(&crate_dir, &output_dir);

    // Configure linking for different build types
    configure_linking();

    // Set up rerun triggers
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=include/");
    println!("cargo:rerun-if-changed=cbindgen.toml");
    println!("cargo:rerun-if-changed=build.rs");
}

fn generate_c_header(crate_dir: &str, output_dir: &PathBuf) {
    // Only generate headers when cbindgen feature is enabled
    #[cfg(feature = "cbindgen")]
    {
        // Check if cbindgen is available and cbindgen.toml exists
        let cbindgen_toml = PathBuf::from(crate_dir).join("cbindgen.toml");
        
        if cbindgen_toml.exists() {
            match cbindgen::Builder::new()
                .with_crate(crate_dir)
                .with_config(cbindgen::Config::from_file(&cbindgen_toml).unwrap())
                .generate()
            {
                Ok(bindings) => {
                    let header_path = output_dir.join("bignum_generated.h");
                    bindings.write_to_file(&header_path);
                    println!("cargo:info=Generated C header: {}", header_path.display());
                }
                Err(e) => {
                    println!("cargo:warning=cbindgen generation failed: {}", e);
                    println!("cargo:warning=Falling back to static header only");
                }
            }
        } else {
            println!("cargo:warning=No cbindgen.toml found, skipping header generation");
        }
    }
    
    #[cfg(not(feature = "cbindgen"))]
    {
        let _ = crate_dir;  // Silence unused variable warning
        let _ = output_dir; // Silence unused variable warning
        println!("cargo:info=cbindgen feature not enabled, skipping header generation");
    }
}

fn copy_static_header(crate_dir: &str, output_dir: &PathBuf) {
    let static_header = PathBuf::from(crate_dir).join("include").join("bignum_ffi.h");
    let target_header = output_dir.join("bignum_ffi.h");
    
    if static_header.exists() {
        std::fs::copy(&static_header, &target_header)
            .expect("Failed to copy static header");
        println!("cargo:info=Copied static header: {}", target_header.display());
    }
}

fn configure_linking() {
    let target = env::var("TARGET").unwrap();
    let profile = env::var("PROFILE").unwrap();
    
    // Configure for embedded targets
    if target.contains("arm") || target.contains("thumbv") {
        println!("cargo:rustc-link-arg=-nostdlib");
        println!("cargo:rustc-cfg=embedded_target");
    }
    
    // Configure for different build profiles
    match profile.as_str() {
        "release" => {
            // Optimize for size in firmware builds
            println!("cargo:rustc-link-arg=-Os");
            println!("cargo:rustc-link-arg=-flto");
        }
        "debug" => {
            // Include debug symbols for development
            println!("cargo:rustc-link-arg=-g");
        }
        _ => {}
    }
    
    // Set library type for C linking with symbol export control
    // Note: symbols.map disabled due to linker conflicts with some toolchains
    // println!("cargo:rustc-cdylib-link-arg=-Wl,--version-script,symbols.map");
}

fn target_dir() -> PathBuf {
    // Get the target directory for build artifacts
    let mut target_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    target_dir.pop(); // Remove hash
    target_dir.pop(); // Remove "out"
    target_dir.pop(); // Remove build script name
    target_dir.pop(); // Remove "build"
    target_dir
}

// Additional build configuration for specific features
#[cfg(feature = "cbindgen")]
fn setup_cbindgen() {
    // This would be called if cbindgen feature is enabled
    println!("cargo:info=Setting up cbindgen for header generation");
}

// Firmware-specific build configuration
#[cfg(feature = "firmware")]
fn setup_firmware_build() {
    println!("cargo:rustc-cfg=firmware_build");
    println!("cargo:rustc-link-arg=-nostdlib");
    println!("cargo:rustc-link-arg=-ffreestanding");
    
    // Ensure no heap allocation
    println!("cargo:rustc-cfg=no_heap");
}

// Test configuration
#[cfg(feature = "test-c-integration")]
fn setup_c_tests() {
    println!("cargo:rustc-link-lib=static=bignum_test");
    println!("cargo:rustc-link-search=native=tests/c");
}
