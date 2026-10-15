//! Progress shared by the native installer, the GUI and headless acceptance tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobPhase {
    Downloading { done: u64, total: u64 },
    Reconnecting { done: u64, total: u64 },
    Verifying,
    Waiting,
    Installing,
    Removing,
}
pub const PAUSED: &str = "paused";
pub const CANCELLED: &str = "cancelled";
pub const INSTALL_CANCELLED: &str = "install cancelled";
