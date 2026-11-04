//! Gives the Windows program its name and version, so the taskbar and the
//! file's Details say "Hall of Worlds" rather than the file's name. Nothing
//! for other targets (the web build, Linux and macOS).

fn main() {
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource
            .set("FileDescription", "Hall of Worlds")
            .set("ProductName", "Hall of Worlds");
        if let Err(error) = resource.compile() {
            // A missing resource compiler only loses the name, so build anyway.
            println!("cargo:warning=No Windows program name: {error}");
        }
    }
}
