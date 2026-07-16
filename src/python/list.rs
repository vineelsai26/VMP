use std::{fs, io};

use super::versions::fetch_available_versions;
use crate::utils::paths::vmp_dir;

pub async fn list_python_versions(
    list_type: String,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut versions = Vec::new();
    if list_type == "all" {
        versions = fetch_available_versions().await?;
    } else if list_type == "installed" || list_type.is_empty() {
        let python_install_path = vmp_dir()?.join("python");

        let entries = match fs::read_dir(&python_install_path) {
            Ok(entries) => entries,
            // No Python installed yet (fresh install): report an empty list
            // instead of panicking.
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(versions),
            Err(error) => return Err(error.into()),
        };

        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();
            if name.starts_with('v') {
                versions.push(name.into_owned());
            }
        }
    } else {
        return Err(
            format!("unknown option {list_type}, please use `all` or `installed` instead").into(),
        );
    }

    Ok(versions)
}
