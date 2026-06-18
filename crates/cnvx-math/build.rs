fn main() {
    // On Arch (and some other distros), LAPACKE is bundled into liblapack.so
    // On Ubuntu/Debian, it's a separate liblapacke.so
    if std::path::Path::new("/usr/lib/liblapacke.so").exists()
        || std::path::Path::new("/usr/lib/x86_64-linux-gnu/liblapacke.so").exists()
    {
        println!("cargo:rustc-link-lib=lapacke");
    } else {
        println!("cargo:rustc-link-lib=lapack");
    }
}
