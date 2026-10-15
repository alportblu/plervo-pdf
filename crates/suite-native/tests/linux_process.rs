#![cfg(target_os = "linux")]
use std::os::unix::{ffi::OsStringExt, process::CommandExt};
struct Probe(std::process::Child);
impl Drop for Probe {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
#[test]
fn a_running_app_with_a_non_utf8_path_is_not_treated_as_stopped() {
    let mut name = format!("/tmp/plervo-nonutf8-{}-", std::process::id()).into_bytes();
    name.push(0xff);
    let path = std::path::PathBuf::from(std::ffi::OsString::from_vec(name));
    let _probe = Probe(std::process::Command::new("/bin/sleep").arg0(&path).arg("30").spawn().unwrap());
    assert!(plervo_suite_native::process::is_running(&path));
}
