//! Store edition: local package inventory and Windows activation, never an external installer.
use std::collections::BTreeMap;

pub const PUBLISHER: &str = "CN=B95B2115-019C-4863-ABF3-046177D1E135";
pub const UPDATES_URI: &str = "ms-windows-store://downloadsandupdates";
pub const SETTINGS_URI: &str = "ms-settings:appsfeatures";

pub struct Product {
    pub id: &'static str,
    pub identity: &'static str,
    pub application: &'static str,
    // Only a LIVE listing belongs here. A reservation is not a published product.
    pub store_id: Option<&'static str>,
}
pub const PRODUCTS: &[Product] = &[
    Product { id: "pdf", identity: "RicardoBonetti.PlervoPDF", application: "PlervoPDF", store_id: Some("9PBWKBZRBJNR") },
    Product { id: "video", identity: "RicardoBonetti.PlervoVideo", application: "PlervoVideo", store_id: Some("9NRKLM46BCDG") },
    Product { id: "cad", identity: "RicardoBonetti.PlervoCAD", application: "PlervoCAD", store_id: Some("9P5CZZSKKG1Q") },
    Product { id: "photo", identity: "RicardoBonetti.PlervoPhoto", application: "PlervoPhoto", store_id: Some("9P5LZ60KSD8N") },
    Product { id: "vector", identity: "RicardoBonetti.PlervoVector", application: "PlervoVector", store_id: Some("9NMWZRJZHDDH") },
    Product { id: "develop", identity: "RicardoBonetti.PlervoDevelop", application: "PlervoDevelop", store_id: Some("9PLTQ9RL45R3") },
    Product { id: "motion", identity: "RicardoBonetti.PlervoMotion", application: "PlervoMotion", store_id: Some("9NSG0HNRW1TF") },
    Product { id: "publish", identity: "RicardoBonetti.PlervoPublish", application: "PlervoPublish", store_id: Some("9N9D719Z3CLR") },
    Product { id: "write", identity: "RicardoBonetti.PlervoWrite", application: "PlervoWrite", store_id: Some("9P3MPXHK0MWQ") },
    Product { id: "slides", identity: "RicardoBonetti.PlervoSlides", application: "PlervoSlides", store_id: Some("9PJP84C61W48") },
    Product { id: "sheets", identity: "RicardoBonetti.PlervoSheets", application: "PlervoSheets", store_id: Some("9NKPX3S5VQ4F") },
    Product { id: "audio", identity: "RicardoBonetti.PlervoAudio", application: "PlervoAudio", store_id: Some("9PJ7F61RGP11") },
    Product { id: "canvas", identity: "RicardoBonetti.PlervoCanvas", application: "PlervoCanvas", store_id: Some("9N51C6Q611QS") },
];

pub fn product(id: &str) -> Result<&'static Product, String> {
    PRODUCTS.iter().find(|p| p.id == id).ok_or_else(|| "This app has no verified Microsoft Store identity yet.".into())
}
pub fn acquisition_uri(id: &str) -> Result<String, String> {
    let store_id = product(id)?.store_id.ok_or("This app is not available in Microsoft Store yet.")?;
    Ok(format!("ms-windows-store://pdp/?ProductId={store_id}"))
}

#[cfg(any(windows, test))]
fn trusted(product: &Product, name: &str, publisher: &str, store_signed: bool, healthy: bool) -> bool {
    name == product.identity && publisher == PUBLISHER && store_signed && healthy
}
pub fn scan() -> Result<BTreeMap<String, String>, String> {
    #[cfg(windows)]
    {
        native::scan().map_err(|e| format!("Windows could not read your installed Plervo apps: {e}"))
    }
    #[cfg(not(windows))]
    Err("The Microsoft Store edition requires Windows.".into())
}
pub fn open(id: &str) -> Result<(), String> {
    let product = product(id)?;
    #[cfg(windows)]
    {
        native::open(product).map_err(|e| format!("Windows could not open this app: {e}"))
    }
    #[cfg(not(windows))]
    {
        let _ = product;
        Err("The Microsoft Store edition requires Windows.".into())
    }
}
/// Fixed protocol allowlist; no arbitrary URLs, shell commands or filesystem execution.
pub fn launch_uri(uri: &str) -> Result<(), String> {
    if uri != UPDATES_URI && uri != SETTINGS_URI && !PRODUCTS.iter().any(|p| acquisition_uri(p.id).as_deref() == Ok(uri)) {
        return Err("This is not an approved Microsoft Store action.".into());
    }
    #[cfg(windows)]
    {
        native::launch_uri(uri).map_err(|e| format!("Windows could not open Microsoft Store or Settings: {e}"))
    }
    #[cfg(not(windows))]
    Err("The Microsoft Store edition requires Windows.".into())
}

#[cfg(windows)]
mod native {
    use super::*;
    use windows::{
        ApplicationModel::{Package, PackageSignatureKind},
        Management::Deployment::PackageManager,
        core::{Error, HSTRING, Result},
    };

    fn installed(product: &Product) -> Result<Option<Package>> {
        let packages = PackageManager::new()?.FindPackagesByUserSecurityIdNamePublisher(
            &HSTRING::new(),
            &HSTRING::from(product.identity),
            &HSTRING::from(PUBLISHER),
        )?;
        for package in packages {
            let id = package.Id()?;
            if trusted(
                product,
                &id.Name()?.to_string_lossy(),
                &id.Publisher()?.to_string_lossy(),
                package.SignatureKind()? == PackageSignatureKind::Store,
                package.Status()?.VerifyIsOK()?,
            ) {
                return Ok(Some(package));
            }
        }
        Ok(None)
    }
    pub(super) fn scan() -> Result<BTreeMap<String, String>> {
        let mut result = BTreeMap::new();
        for product in PRODUCTS {
            if let Some(package) = installed(product)? {
                let v = package.Id()?.Version()?;
                result.insert(product.id.into(), format!("{}.{}.{}.{}", v.Major, v.Minor, v.Build, v.Revision));
            }
        }
        Ok(result)
    }
    pub(super) fn open(product: &Product) -> Result<()> {
        let package = installed(product)?
            .ok_or_else(|| Error::new(windows::core::HRESULT(0x80070490u32 as i32), "App is not installed from Microsoft Store."))?;
        let aumid = format!("{}!{}", package.Id()?.FamilyName()?, product.application);
        for entry in package.GetAppListEntriesAsync()?.join()? {
            if entry.AppUserModelId()?.to_string_lossy() == aumid && entry.LaunchAsync()?.join()? {
                return Ok(());
            }
        }
        Err(Error::new(windows::core::HRESULT(0x80004005u32 as i32), "Windows did not launch the registered app."))
    }
    pub(super) fn launch_uri(uri: &str) -> Result<()> {
        let uri = windows::Foundation::Uri::CreateUri(&HSTRING::from(uri))?;
        if windows::System::Launcher::LaunchUriAsync(&uri)?.join()? {
            Ok(())
        } else {
            Err(Error::new(windows::core::HRESULT(0x80004005u32 as i32), "The Microsoft Store or Settings app is unavailable."))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inventory_requires_exact_publisher_product_store_signature_and_health() {
        let pdf = product("pdf").unwrap();
        assert!(trusted(pdf, pdf.identity, PUBLISHER, true, true));
        assert!(!trusted(pdf, "RicardoBonetti.PlervoVideo", PUBLISHER, true, true));
        assert!(!trusted(pdf, pdf.identity, "CN=Another publisher", true, true));
        assert!(!trusted(pdf, pdf.identity, PUBLISHER, false, true));
        assert!(!trusted(pdf, pdf.identity, PUBLISHER, true, false));
        assert!(!trusted(pdf, &format!("{}-copy", pdf.identity), PUBLISHER, true, true));
    }
    #[test]
    fn only_live_products_have_acquisition_links() {
        assert_eq!(acquisition_uri("pdf").unwrap(), "ms-windows-store://pdp/?ProductId=9PBWKBZRBJNR");
        assert_eq!(acquisition_uri("video").unwrap(), "ms-windows-store://pdp/?ProductId=9NRKLM46BCDG");
        assert_eq!(acquisition_uri("cad").unwrap(), "ms-windows-store://pdp/?ProductId=9P5CZZSKKG1Q");
        assert_eq!(acquisition_uri("photo").unwrap(), "ms-windows-store://pdp/?ProductId=9P5LZ60KSD8N");
        assert_eq!(acquisition_uri("canvas").unwrap(), "ms-windows-store://pdp/?ProductId=9N51C6Q611QS");
        for id in ["unknown", "../pdf", "pdf&ProductId=other", ""] {
            assert!(acquisition_uri(id).is_err());
        }
    }
    #[test]
    fn arbitrary_shell_urls_and_external_installers_are_refused() {
        for uri in [
            "file:///C:/malware.exe",
            "https://github.com/alportblu/plervo-pdf/releases",
            "ms-windows-store://pdp/?ProductId=OTHER",
            "ms-settings:other",
            "cmd.exe",
        ] {
            assert_eq!(launch_uri(uri).unwrap_err(), "This is not an approved Microsoft Store action.");
        }
    }
}
