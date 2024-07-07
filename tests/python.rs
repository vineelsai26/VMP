use vmp::python::{install, list, uninstall, usage};

#[tokio::test]
async fn test_python_3_12() {
    let _result = install::install_python("3.12".to_string()).await;
    let _result = usage::use_python("3.12".to_string()).await;
    let _result = list::list_python_versions("installed".to_string()).await;
    let _result = list::list_python_versions("all".to_string()).await;
    let _result = uninstall::uninstall_python("3.12".to_string()).await;
}
