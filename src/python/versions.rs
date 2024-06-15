use reqwest::{self, header::USER_AGENT};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct GitTag {
    #[serde(rename = "ref")]
    tag: String,
}

pub async fn fetch_available_versions() -> Result<Vec<String>, Box<dyn std::error::Error>> {
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

pub async fn get_latest_minor_of_version(
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
