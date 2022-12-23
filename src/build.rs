fn main() {
    #[cfg(target_os="macos")]
    println!("cargo:rustc-link-search=/opt/homebrew/lib");

    #[cfg(target_os="linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
}