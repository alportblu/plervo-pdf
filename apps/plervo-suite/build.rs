//! The Windows launcher carries its own icon and version, with no elevation request.
fn main() {
    println!("cargo:rerun-if-changed=../../assets/products/suite/icon.ico");
    println!("cargo:rerun-if-changed=windows.manifest");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut resource = winresource::WindowsResource::new();
    resource
        .set_icon("../../assets/products/suite/icon.ico")
        .set_manifest(include_str!("windows.manifest"))
        .set("ProductName", "Plervo Suite")
        .set("ProductVersion", env!("CARGO_PKG_VERSION"))
        .set("FileDescription", "Plervo Suite application manager")
        .set("CompanyName", "Plervo")
        .set("OriginalFilename", "plervo-suite.exe")
        .set("InternalName", "plervo-suite");
    if let Err(error) = resource.compile() {
        println!("cargo::error=embedding Plervo Suite Windows resources failed: {error}");
    }
}
