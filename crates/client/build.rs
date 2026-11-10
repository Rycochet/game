//! Gives the Windows program its name, version and icon, so the taskbar and
//! the file's Details say "Hall of Worlds" rather than the file's name, and
//! Explorer shows the HW icon (drawn by `cargo xtask app-icon`). Nothing for
//! other targets (the web build, Linux and macOS).

const ICON: &str = "assets/app_icon/icon.ico";

fn main() {
    println!("cargo:rerun-if-changed={ICON}");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource
            .set("FileDescription", "Hall of Worlds")
            .set("ProductName", "Hall of Worlds")
            // Id 1, which the window's icon is loaded by (`app_icon`).
            .set_icon(ICON);
        if let Err(error) = resource.compile() {
            // A missing resource compiler only loses the name and icon, so build
            // anyway.
            println!("cargo:warning=No Windows program name or icon: {error}");
        }
    }
}
