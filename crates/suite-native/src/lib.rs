//! Installation code adapted from craft-launcher 0.4.1; see docs/suite-provenance.md.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
pub mod desktop_entry;
pub mod detect;
pub mod http;
pub mod install;
#[cfg(unix)]
pub mod managed_linux;
pub mod manager;
pub mod manifest;
pub mod packages;
pub mod paths;
pub mod process;
pub mod proxy;
pub mod queue;
#[cfg(test)]
mod test_catalog;
pub mod updates;
pub fn now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}
