use homedir::get_my_home;
use sigstore_verify::{
    trust_root::{TrustedRoot, SIGSTORE_PRODUCTION_TRUSTED_ROOT},
    types::Bundle,
    verify, VerificationPolicy,
};
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
    let bundle_path = format!("{file_path}.sigstore");
    let bundle_url = format!("{url}.sigstore");

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
            download_file(url.clone(), &file_path).await?;
        } else {
            println!(
                "File already exists using already downloaded file from {}",
                file_path
            )
        }
        if !Path::new(&bundle_path).exists() {
            println!("Downloading Python Sigstore bundle from {}", bundle_url);
            download_file(bundle_url, &bundle_path).await?;
        }
        verify_python_release(&version, &file_path, &bundle_path)?;

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

fn verify_python_release(
    version: &str,
    artifact_path: &str,
    bundle_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (identity, issuer) = python_release_identity(version)?;
    let artifact = std::fs::read(artifact_path)?;
    let bundle_json = std::fs::read_to_string(bundle_path)?;
    let bundle = Bundle::from_json(&bundle_json)?;
    let trusted_root = TrustedRoot::from_json(SIGSTORE_PRODUCTION_TRUSTED_ROOT)?;
    let policy = VerificationPolicy::default()
        .require_identity(identity)
        .require_issuer(issuer);
    verify(artifact.as_slice(), &bundle, &policy, &trusted_root).map_err(|error| {
        io::Error::other(format!(
            "Python release signature verification failed: {error}"
        ))
    })?;
    println!("Verified Python {} Sigstore identity {}", version, identity);
    Ok(())
}

fn python_release_identity(version: &str) -> Result<(&'static str, &'static str), io::Error> {
    let normalized = version.trim_start_matches('v');
    let parts = normalized.split('.').collect::<Vec<_>>();
    let minor = parts
        .get(1)
        .and_then(|value| value.parse::<u32>().ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid Python version"))?;
    match minor {
        7 => Ok(("nad@python.org", "https://github.com/login/oauth")),
        8 | 9 => Ok(("lukasz@langa.pl", "https://github.com/login/oauth")),
        10 | 11 => Ok(("pablogsal@python.org", "https://accounts.google.com")),
        12 | 13 => Ok(("thomas@python.org", "https://accounts.google.com")),
        14 | 15 => Ok(("hugo@python.org", "https://github.com/login/oauth")),
        16 | 17 => Ok(("savannah@python.org", "https://github.com/login/oauth")),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Python {normalized} has no pinned CPython Sigstore release identity"),
        )),
    }
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

#[cfg(test)]
mod tests {
    use super::python_release_identity;

    #[test]
    fn release_identity_is_pinned_by_minor_series() {
        assert_eq!(
            python_release_identity("v3.14.6").unwrap(),
            ("hugo@python.org", "https://github.com/login/oauth")
        );
        assert!(python_release_identity("3.6.15").is_err());
    }
}
