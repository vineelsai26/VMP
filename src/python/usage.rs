use super::versions::resolve_version;
use crate::utils::paths::vmp_dir;
use std::{fs::File, io::Write};

pub async fn use_python(version: String) -> Result<(), Box<dyn std::error::Error>> {
    let version = resolve_version(&version).await?;

    let vmp_path = vmp_dir()?;
    let vmp_path_python_path = vmp_path.join("python").join(&version).join("bin");

    // Confirm the requested version is installed *before* touching the
    // python_version file, so a `use` of a missing version cannot clobber the
    // user's currently selected version.
    if !vmp_path_python_path.exists() {
        return Err(format!(
            "Python version {version} not installed, please install it with command \n `vmp install {version}`"
        )
        .into());
    }

    std::fs::create_dir_all(&vmp_path)?;
    let vmp_python_version = vmp_path.join("python_version");
    let mut file = File::create(&vmp_python_version)?;

    println!("Using Python Version {}", version);
    file.write_all(version.as_bytes())?;

    Ok(())
}
