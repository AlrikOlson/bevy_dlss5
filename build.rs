// Links the NGX core loader (nvsdk_ngx_d.lib) from the NVIDIA DLSS SDK.
// Requires the DLSS_SDK environment variable (path to a checkout of
// https://github.com/NVIDIA/DLSS). No bindgen / clang / Vulkan SDK needed —
// the small NGX surface this crate uses is declared by hand.
fn main() {
    println!("cargo:rerun-if-env-changed=DLSS_SDK");
    let sdk = match std::env::var("DLSS_SDK") {
        Ok(s) => s,
        Err(_) => {
            println!("cargo:warning=DLSS_SDK not set; bevy_dlss5 needs it to link nvsdk_ngx_d.lib. See the README.");
            return;
        }
    };
    println!("cargo:rustc-link-search=native={sdk}/lib/Windows_x86_64/x64");
    println!("cargo:rustc-link-lib=static=nvsdk_ngx_d");
    for lib in ["advapi32", "user32", "shell32", "ole32", "delayimp"] {
        println!("cargo:rustc-link-lib=dylib={lib}");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
