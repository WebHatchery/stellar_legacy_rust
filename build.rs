fn main() {
    println!("cargo:rerun-if-changed=assets/packaging/stellar_legacy.ico");

    let windows_target = std::env::var("TARGET")
        .map(|target| target.contains("windows"))
        .unwrap_or(false);
    if cfg!(windows) && windows_target {
        let version = env!("CARGO_PKG_VERSION");
        let mut parts = version
            .split('.')
            .map(|part| part.parse::<u64>().unwrap_or(0));
        let packed = (parts.next().unwrap_or(0) << 48)
            | (parts.next().unwrap_or(0) << 32)
            | (parts.next().unwrap_or(0) << 16);
        let mut resource = winres::WindowsResource::new();
        resource
            .set_icon("assets/packaging/stellar_legacy.ico")
            .set_language(0x0409)
            .set_version_info(winres::VersionInfo::FILEVERSION, packed)
            .set_version_info(winres::VersionInfo::PRODUCTVERSION, packed)
            .set("FileDescription", "Stellar Legacy")
            .set("ProductName", "Stellar Legacy")
            .set("OriginalFilename", "stellar_legacy.exe")
            .set("InternalName", "stellar_legacy")
            .set("ProductVersion", version)
            .set("FileVersion", version);
        resource
            .compile()
            .expect("Windows application resource compilation failed");
    }
}
