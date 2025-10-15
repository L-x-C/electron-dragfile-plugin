extern crate napi_build;

fn main() {
    // Setup basic napi configuration
    napi_build::setup();

    // On macOS, link against required frameworks
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=CoreGraphics");
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=Foundation");

        // Set deployment target for better compatibility
        println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.13");
    }
}