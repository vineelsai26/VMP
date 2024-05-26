use flate2::read::GzDecoder;
use futures_util::StreamExt;
use homedir::get_my_home;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{self, header::USER_AGENT};
use serde::{Deserialize, Serialize};
use std::{
    cmp::min,
    fs::File,
    io::Write,
    path::Path,
    process::{exit, Command},
};
use tar::Archive;

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

async fn download_file(url: String, file_path: String) -> Result<(), Box<dyn std::error::Error>> {
    let res = reqwest::get(url.clone()).await?;

    let download_size = res.content_length().expect("");

    let pb = ProgressBar::new(download_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
            .progress_chars("#>-")
    );
    pb.set_message("Downloading ".to_string() + &url);

    // let mut data = Cursor::new(res.bytes().await?);
    // let mut out = File::create(file_path).expect("failed to create file");
    // copy(&mut data, &mut out).expect("failed to copy content");

    let mut file = File::create(file_path.clone())
        .or(Err(format!("Failed to create file '{}'", file_path)))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write_all(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        let new = min(downloaded + (chunk.len() as u64), download_size);
        downloaded = new;
        pb.set_position(new);
    }

    Ok(())
}

fn extract_tar_gz(source: String, dest: String, prefix: String) -> Result<(), std::io::Error> {
    let tar_gz = File::open(source)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    // let _ = archive.unpack(dest);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        let stripped_path = path.strip_prefix(&prefix).unwrap();
        // let mut dst = File::create(&stripped_path)?;
        let dst = dest.clone() + "/" + stripped_path.to_str().unwrap();

        println!("{:?}", dst);
        _ = entry.unpack(dst);
    }

    Ok(())
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

    println!("Installing Python Version {:?}", version);

    let url = "https://www.python.org/ftp/python/".to_owned()
        + &version.replace("v", "")
        + "/Python-"
        + &version.replace("v", "")
        + ".tgz";

    println!("Downloading Python from {:?}", url);

    let vmp_path = get_my_home().unwrap().unwrap().as_path().join(".vmp");
    let download_cache_path = vmp_path.join("cache");
    let _ = std::fs::create_dir_all(download_cache_path.clone());
    let file_name = url.split("/").last().unwrap().to_string();
    let file_path = download_cache_path
        .join(file_name.clone())
        .to_str()
        .unwrap()
        .to_string(); // TODO: too crazy fix it

    let already_downloaded = Path::new(&file_path).exists();
    if !already_downloaded {
        download_file(url, file_path.clone()).await?;
    } else {
        println!(
            "File already exists using already downloaded file from {:?}",
            file_path
        )
    }

    let extract_path = download_cache_path
        .join(version.clone())
        .to_str()
        .unwrap()
        .to_string();

    let _ = std::fs::create_dir_all(extract_path.clone());
    let _ = extract_tar_gz(
        file_path,
        extract_path.clone(),
        file_name.replace(".tgz", ""),
    );

    let install_path = vmp_path.join("python").join(version).to_str().unwrap().to_string();

    let build_args = format!("--prefix={} --enable-optimizations && make && make altinstall", install_path);

    println!("{:?}", extract_path);

    let _ = Command::new("bash")
        .arg("-c")
        .arg(format!("cd {} && ./configure {}", extract_path, build_args)).spawn().unwrap().wait();

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
