use homedir::get_my_home;
use std::fs::remove_dir_all;

use crate::python::versions::get_latest_minor_of_version;

pub async fn uninstall_python(mut version: String) -> Result<(), Box<dyn std::error::Error>> {
    version = version.replace("v", "");
    let version_string = version.split(".").filter(|&i| i != "").collect::<Vec<_>>();

    if version_string.len() == 1 {
        version =
            get_latest_minor_of_version(version_string[0].to_string(), "".to_string()).await?;
    } else if version_string.len() == 2 {
        version = get_latest_minor_of_version(
            version_string[0].to_string(),
            version_string[1].to_string(),
        )
        .await?;
    } else if version_string.len() == 3 {
        version = version
    } else {
        panic!("error invalid version number")
    }

    if !version.contains("v") {
        version = "v".to_owned() + &version
    }

    let python_path = get_my_home()
        .unwrap()
        .unwrap()
        .as_path()
        .join(".vmp")
        .join("python")
        .join(&version);

    if python_path.exists() {
        remove_dir_all(python_path)?;
        println!("Uninstall for python version {} is successfull", version)
    } else {
        println!("Python version {} is not installed", version)
    }

    Ok(())
}
