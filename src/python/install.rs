use homedir::get_my_home;
use std::{io, path::Path, process::Command};

use crate::{
    python::versions::get_latest_minor_of_version,
    utils::{archive::extract_tar_gz, network::download_file},
};

pub async fn install_python(mut version: String) -> Result<(), Box<dyn std::error::Error>> {
    version = version.replace("v", "");
    let version_string = version
        .split('.')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if version_string.len() == 1 {
        version =
            get_latest_minor_of_version(version_string[0].to_string(), "".to_string()).await?;
    } else if version_string.len() == 2 {
        version = get_latest_minor_of_version(
            version_string[0].to_string(),
            version_string[1].to_string(),
        )
        .await?;
    } else if version_string.len() != 3 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid version number").into());
    }

    if !version.contains("v") {
        version = "v".to_owned() + &version
    }

    let url = "https://www.python.org/ftp/python/".to_owned()
        + &version.replace("v", "")
        + "/Python-"
        + &version.replace("v", "")
        + ".tgz";

    let vmp_path = get_my_home().unwrap().unwrap().as_path().join(".vmp");
    let download_cache_path = vmp_path.join("cache");
    let _ = std::fs::create_dir_all(&download_cache_path);
    let file_name = url.split("/").last().unwrap().to_string();
    let file_path = download_cache_path
        .join(&file_name)
        .to_str()
        .unwrap()
        .to_string();

    let extract_path = download_cache_path
        .join(&version)
        .to_str()
        .unwrap()
        .to_string();

    let install_path = vmp_path
        .join("python")
        .join(&version)
        .to_str()
        .unwrap()
        .to_string();

    println!("Installing Python Version {} at {}", version, extract_path);

    let vmp_python_bin_path = vmp_path.join("python").join(&version).join("bin");

    if vmp_python_bin_path.exists() {
        println!("Python version {} already installed", version)
    } else {
        let already_downloaded = Path::new(&file_path).exists();
        if !already_downloaded {
            println!("Downloading Python from {}", url);
            download_file(url, &file_path).await?;
        } else {
            println!(
                "File already exists using already downloaded file from {}",
                file_path
            )
        }

        extract_tar_gz(file_path, &extract_path, file_name.replace(".tgz", ""))?;

        run_build_step(
            Command::new("./configure")
                .current_dir(&extract_path)
                .arg(format!("--prefix={install_path}"))
                .arg("--enable-optimizations"),
            "configure Python",
        )?;
        run_build_step(
            Command::new("make").current_dir(&extract_path),
            "build Python",
        )?;
        run_build_step(
            Command::new("make")
                .current_dir(&extract_path)
                .arg("altinstall"),
            "install Python",
        )?;
    }

    Ok(())
}

fn run_build_step(command: &mut Command, description: &str) -> io::Result<()> {
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "failed to {description}: process exited with {status}"
        )))
    }
}
