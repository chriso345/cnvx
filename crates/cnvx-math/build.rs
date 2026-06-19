fn main() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    if cfg!(target_os = "windows") {
        println!("cargo:error=Windows is not currently supported.");
        std::process::exit(1);
    }

    if target_arch == "wasm32" {
        println!(
            "cargo:warning=WebAssembly target detected, skipping lapack probe, using custom implementations."
        );
    } else {
        probe_lapack();
    }
}

fn probe_lapack() {
    // Try lapacke first, fall back to lapack.
    if pkg_config::probe_library("lapacke").is_err() {
        pkg_config::probe_library("lapack")
            .expect("could not find lapack or lapacke via pkg-config");
    }

    // Add blas here as a "just in case"
    pkg_config::probe_library("blas").expect("could not find blas via pkg-config");
}
