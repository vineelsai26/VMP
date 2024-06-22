use vmp::python::{install, list, uninstall, usage};

#[tokio::test]
async fn test_python_install() {
    let _result = install::install_python("3.12".to_string()).await;
}

#[tokio::test]
async fn test_python_use() {
    let _result = usage::use_python("3.12".to_string()).await;
}

#[tokio::test]
async fn test_python_uninstall() {
    let _result = uninstall::uninstall_python("3.12".to_string()).await;
}

#[tokio::test]
async fn test_python_list() {
    let _result = list::list_python_versions("3.12".to_string()).await;
}
