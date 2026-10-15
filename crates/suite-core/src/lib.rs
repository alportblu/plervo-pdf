//! Plervo Suite: adapted from craft-launcher 0.4.1 (see docs/suite-provenance.md).
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
pub mod catalog;
pub mod distro;
pub mod platform;
pub mod protocol;
pub mod release;
pub mod status;
pub mod time;
pub mod version;
pub use catalog::{AppGroup, CraftApp, SUITE};
pub use platform::{Arch, Os, PackageKind, Platform};
pub use release::{Asset, Release};
pub use status::{AppStatus, InstallSource, Installed, Latest};
