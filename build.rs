//! Windows: embed the app icon and version info into the .exe.
fn main() {
    println!("cargo:rerun-if-changed=Resources/claude-companion.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("Resources/claude-companion.ico");
        res.set("ProductName", "Claude Companion");
        res.set("FileDescription", "Claude Companion");
        if let Err(e) = res.compile() {
            println!("cargo:warning=no icon in the .exe: {e}");
        }
    }
}
