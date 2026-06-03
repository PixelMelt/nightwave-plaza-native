fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/assets/icons/nightwave-plaza.ico");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("src/assets/icons/nightwave-plaza.ico");
        res.compile().expect("failed to embed Windows resource");
    }
}
