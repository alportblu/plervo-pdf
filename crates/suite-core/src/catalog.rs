//! The shipped Plervo catalog contains only products with reviewed releases.
use serde::{Deserialize, Serialize};

/// Which generation of the suite an app belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AppGroup {
    /// The seven original apps.
    Classic,
    /// The six apps added after them.
    New,
}

impl AppGroup {
    pub fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::New => "New",
        }
    }
}

/// One app in the suite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CraftApp {
    /// Machine name: repo, binary, package and asset stem (`photocraft`).
    pub id: &'static str,
    /// Display name (`PhotoCraft`).
    pub name: &'static str,
    /// What it's for, in a few words (English; the UI translates it by app id).
    pub tagline: &'static str,
    pub group: AppGroup,
    /// Brand colour (sRGB), for surfaces: getartcraft.com's `--app`.
    pub color: [u8; 3],
    /// The brand colour for text on a light background (the site's light `--app-ink`).
    pub ink_light: [u8; 3],
    /// The brand colour for text on a dark background (the site's dark `--app-ink`).
    pub ink_dark: [u8; 3],
    /// GitHub `owner/repo`.
    pub repo: &'static str,
    /// Older names its release assets may carry (PdfCraft shipped as PrintCraft until 0.2.1).
    pub former_ids: &'static [&'static str],
    /// Older display names its installers registered (PdfCraft's MSI said PrintCraft until 0.2.1).
    pub former_names: &'static [&'static str],
}

impl CraftApp {
    /// `com.plervo.<product>`: macOS bundle id, Flatpak id and Linux desktop-file id.
    pub fn bundle_id(&self) -> String {
        format!("com.plervo.{}", self.id.strip_prefix("plervo-").unwrap_or(self.id))
    }

    pub fn releases_url(&self) -> String {
        format!("https://github.com/{}/releases", self.repo)
    }

    pub fn repo_url(&self) -> String {
        format!("https://github.com/{}", self.repo)
    }

    /// The app's page on getartcraft.com.
    pub fn website(&self) -> String {
        "https://plervo.com/".to_string()
    }

    /// Asset stems this app's releases may use, current first.
    pub fn stems(&self) -> impl Iterator<Item = &'static str> + '_ {
        std::iter::once(self.id).chain(self.former_ids.iter().copied())
    }

    /// Display names this app's installers may have registered, current first.
    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        std::iter::once(self.name).chain(self.former_names.iter().copied())
    }

    pub fn by_id(id: &str) -> Option<&'static CraftApp> {
        SUITE.iter().find(|a| a.id == id)
    }
}

pub const PDF: CraftApp = CraftApp {
    id: "plervo-pdf",
    name: "Plervo PDF",
    tagline: "A capable workspace for your documents",
    group: AppGroup::Classic,
    color: [190, 224, 64],
    ink_light: [32, 51, 32],
    ink_dark: [202, 232, 120],
    repo: "alportblu/plervo-pdf",
    former_ids: &[],
    former_names: &[],
};
pub const SUITE: [CraftApp; 1] = [PDF];
pub const LAUNCHER: CraftApp = CraftApp { id: "plervo-suite", name: "Plervo Suite", tagline: "Your apps. One place.", ..PDF };
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_our_published_products_and_names_are_recognized() {
        assert_eq!(SUITE.len(), 1);
        assert_eq!(PDF.bundle_id(), "com.plervo.pdf");
        assert_eq!(LAUNCHER.bundle_id(), "com.plervo.suite");
        assert!(CraftApp::by_id("pdfcraft").is_none());
        assert_eq!(PDF.former_ids.len(), 0);
        assert_eq!(PDF.repo, "alportblu/plervo-pdf");
    }
}
