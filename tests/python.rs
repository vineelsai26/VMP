use vmp::python::{install, list, uninstall, usage};

// End-to-end: resolves versions via the GitHub API (unauthenticated, so it is
// rate-limited on shared CI runner IPs) and actually downloads/installs Python.
// Not suitable for unattended CI — run locally with `cargo test -- --ignored`.
#[tokio::test]
#[ignore = "e2e: hits the rate-limited GitHub API and installs Python over the network"]
async fn test_python_3_12() {
    let _result = install::install_python("3.12".to_string()).await;
    let _result = usage::use_python("3.12".to_string()).await;
    let _result = list::list_python_versions("installed".to_string()).await;
    let _result = list::list_python_versions("all".to_string()).await;
    let _result = uninstall::uninstall_python("3.12".to_string()).await;
}
