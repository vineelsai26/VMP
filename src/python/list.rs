use std::fs;

use super::versions::fetch_available_versions;
use homedir::get_my_home;

pub async fn list_python_versions(
    list_type: String,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut versions = Vec::new();
    if list_type == "all" {
        versions = fetch_available_versions().await?;
    } else if list_type == "installed" || list_type == "" {
        let python_install_path = get_my_home()
            .unwrap()
            .unwrap()
            .as_path()
            .join(".vmp")
            .join("python");

        for version in fs::read_dir(python_install_path.as_path()).unwrap() {
            if version
                .as_ref()
                .unwrap()
                .file_name()
                .to_str()
                .unwrap()
                .to_string()
                .starts_with("v")
            {
                versions.push(version.unwrap().file_name().to_str().unwrap().to_string());
            }
        }
    } else {
        panic!(
            "unknown option {}, please use `all` or `installed` instead",
            list_type
        )
    }

    Ok(versions)
}
