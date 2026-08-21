use std::env;
use std::process::exit;

fn main() {
    println!("cargo:rerun-if-changed=src/sparse/mumps_shim.c");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    if target_os == "windows" {
        println!("cargo:error=Windows is not currently supported.");
        exit(1);
    }

    if target_arch == "wasm32" {
        println!(
            "cargo:warning=WebAssembly target detected, skipping native solver dependencies."
        );
        return;
    }

    probe_openblas();
    probe_umfpack();
}

fn probe_openblas() {
    pkg_config::Config::new()
        .cargo_metadata(true)
        .probe("openblas")
        .unwrap_or_else(|e| {
            panic!(
                "could not find OpenBLAS via pkg-config: {e}\n\
                 Install OpenBLAS and pkg-config, e.g.:\n\
                 Arch:   pacman -S openblas pkgconf\n\
                 Ubuntu: apt install libopenblas-dev pkg-config\n\
                 Brew:  brew install openblas pkg-config"
            )
        });
}

fn probe_umfpack() {
    let found = pkg_config::Config::new()
        .cargo_metadata(true)
        .probe("umfpack")
        .or_else(|_| pkg_config::Config::new().cargo_metadata(true).probe("UMFPACK"));

    if let Err(e) = found {
        panic!(
            "could not find UMFPACK via pkg-config: {e}\n\
             Install SuiteSparse, e.g.:\n\
             Arch:   pacman -S suitesparse\n\
             Ubuntu: apt install libsuitesparse-dev\n\
             Brew:  brew install suite-sparse"
        );
    }
}
