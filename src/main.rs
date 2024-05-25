use reqwest::{self, header::USER_AGENT};
use serde::{Deserialize, Serialize};
use std::process::exit;

fn help() {
    println!("Usage: vmn python <command> [version]
Commands:
	python install [version]        			Install a specific version of python or latest or lts version of python (default: lts)
	python use [version]            			Use a specific version of python
	python list [type]              			List all versions, installed versions or lts versions
	python uninstall [version]      			Uninstall a specific version of python
	python help                     			Print this help section
Examples:
	vmn python install latest       			Install the latest version of python
	vmn python use latest           			Use the latest version of python
	vmn python install 3.11         			Install a specific version of python
	vmn python use 3.11		          			Use a specific version of python
	vmn python list all             			List all versions of python
	vmn python list installed       			List installed versions of python
	vmn python uninstall all        			Uninstall all versions of python
	vmn python help                 			Print this help");
    exit(0);
}

fn use_python(version: String) {
    println!("{:?}", version);
}

#[derive(Debug, Serialize, Deserialize)]
struct GitTag {
    #[serde(rename = "ref")]
    tag: String,
}

async fn fetch_available_versions() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let res = client
        .get("https://api.github.com/repos/python/cpython/git/matching-refs/tags/v")
        .header(USER_AGENT, "reqwest rust")
        .send()
        .await?;
    let mut versions: Vec<String> = [].to_vec();

    if res.status() == 200 {
        let tags: Vec<GitTag> = res.json().await?;
        for tag in tags {
            let version = tag.tag.replace("refs/tags/", "");
            if !version.contains("a") && !version.contains("b") && !version.contains("c") {
                versions.push(version);
            }
        }
    }

    versions.reverse();

    return Ok(versions);
}

async fn get_latest_minor_of_version(
    major: String,
    minor: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let versions = fetch_available_versions().await?;

    for version in versions {
        let version_vec = version.split(".").filter(|&i| i != "").collect::<Vec<_>>();

        if version_vec[0] == "v".to_string() + &major && version_vec[1] == minor {
            return Ok(version);
        }
    }

    panic!("Unable to find suitable version");
}

async fn install_python(mut version: String) -> Result<(), Box<dyn std::error::Error>> {
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

    println!("{:?}", version);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = std::env::args().nth(1).expect("Not enough args");

    if cmd == "help" {
        help()
    } else if cmd == "version" {
        println!("dev");
        exit(0);
    } else if cmd == "setup" {
        println!("setup");
        exit(0);
    } else if cmd == "env" {
        println!("env");
        exit(0);
    }

    let version = std::env::args().nth(2).expect("Not enough args");

    if cmd == "install" {
        install_python(version).await?;
    } else if cmd == "use" {
        use_python(version);
    }

    Ok(())
}
