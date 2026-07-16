use std::fs::remove_dir_all;

use crate::python::versions::resolve_version;
use crate::utils::paths::vmp_dir;

pub async fn uninstall_python(version: String) -> Result<(), Box<dyn std::error::Error>> {
    let version = resolve_version(&version).await?;

    let python_path = vmp_dir()?.join("python").join(&version);

    if python_path.exists() {
        remove_dir_all(python_path)?;
        println!("Uninstall for python version {} is successfull", version)
    } else {
        return Err(format!("Python version {version} is not installed").into());
    }

    Ok(())
}
