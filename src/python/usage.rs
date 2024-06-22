use super::versions::get_latest_minor_of_version;
use homedir::get_my_home;
use std::{fs::File, io::Write};

pub async fn use_python(mut version: String) -> Result<(), Box<dyn std::error::Error>> {
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

    let vmp_path = get_my_home().unwrap().unwrap().as_path().join(".vmp");
    let _ = std::fs::create_dir_all(&vmp_path);

    let vmp_python_version = vmp_path
        .join("python_version")
        .to_str()
        .unwrap()
        .to_string();
    let mut file = File::create(vmp_python_version)?;

    let vmp_path_python_path = vmp_path.join("python").join(&version).join("bin");

    if vmp_path_python_path.exists() {
        println!("Using Python Version {}", version);
        file.write_all(version.as_bytes())?;
    } else {
        println!(
            "Python version {} not installed, please install it with command \n `vmp install {}`",
            version, version
        )
    }

    Ok(())
}
