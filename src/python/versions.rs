use reqwest::{self, header::USER_AGENT, StatusCode};
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

    let status = res.status();
    if !status.is_success() {
        if status == StatusCode::FORBIDDEN || status == StatusCode::TOO_MANY_REQUESTS {
            return Err(format!(
                "GitHub API rate limit reached ({status}) while listing Python versions; \
                 please retry later or authenticate to raise the limit"
            )
            .into());
        }
        return Err(
            format!("GitHub API request for Python versions failed with status {status}").into(),
        );
    }

    let tags: Vec<GitTag> = res.json().await?;
    let mut versions: Vec<String> = Vec::new();
    for tag in tags {
        let version = tag.tag.replace("refs/tags/", "");
        if !version.contains("a") && !version.contains("b") && !version.contains("c") {
            versions.push(version);
        }
    }

    versions.reverse();

    Ok(versions)
}

/// Parse a `vX.Y.Z` release tag into its numeric components. Returns `None` for
/// tags that are not a plain three-part final release (e.g. missing a minor or
/// patch component), so callers never index into a short vector.
fn parse_version(version: &str) -> Option<[u64; 3]> {
    let normalized = version.trim_start_matches('v');
    let parts = normalized
        .split('.')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() != 3 {
        return None;
    }
    let major = parts[0].parse().ok()?;
    let minor = parts[1].parse().ok()?;
    let patch = parts[2].parse().ok()?;
    Some([major, minor, patch])
}

/// Resolve a user-supplied version selector into a concrete `vX.Y.Z` string.
///
/// Handles the advertised aliases (`latest`/`lts`), single-major selectors
/// (`3`), major.minor selectors (`3.13`), and fully specified versions
/// (`3.13.1`). Returns an `Err` instead of panicking for empty or malformed
/// input so callers can print a clean CLI error.
pub async fn resolve_version(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let normalized = input.trim().trim_start_matches('v');
    let lowered = normalized.to_lowercase();

    if lowered == "latest" || lowered == "lts" {
        return get_latest_version().await;
    }

    let parts = normalized
        .split('.')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    match parts.len() {
        1 => get_latest_minor_of_version(parts[0].to_string(), String::new()).await,
        2 => get_latest_minor_of_version(parts[0].to_string(), parts[1].to_string()).await,
        3 => Ok(format!("v{normalized}")),
        _ => Err(format!("invalid Python version '{input}'").into()),
    }
}

/// Return the highest available final release across all series.
pub async fn get_latest_version() -> Result<String, Box<dyn std::error::Error>> {
    let versions = fetch_available_versions().await?;

    let mut best: Option<([u64; 3], String)> = None;
    for version in versions {
        if let Some(nums) = parse_version(&version) {
            let replace = match &best {
                Some((current, _)) => nums > *current,
                None => true,
            };
            if replace {
                best = Some((nums, version));
            }
        }
    }

    best.map(|(_, version)| version)
        .ok_or_else(|| "Unable to find any available Python version".into())
}

/// Return the highest available patch release for the given `major` (and
/// optionally `minor`) series. An empty `minor` matches any minor within the
/// major series, supporting single-major selectors like `3`.
pub async fn get_latest_minor_of_version(
    major: String,
    minor: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let versions = fetch_available_versions().await?;
    let major = major.replace("v", "");

    let mut best: Option<([u64; 3], String)> = None;
    for version in versions {
        let Some(nums) = parse_version(&version) else {
            continue;
        };

        if nums[0].to_string() != major {
            continue;
        }
        if !minor.is_empty() && nums[1].to_string() != minor {
            continue;
        }

        let replace = match &best {
            Some((current, _)) => nums > *current,
            None => true,
        };
        if replace {
            best = Some((nums, version));
        }
    }

    best.map(|(_, version)| version).ok_or_else(|| {
        let requested = if minor.is_empty() {
            major.clone()
        } else {
            format!("{major}.{minor}")
        };
        format!("Unable to find a suitable Python version for '{requested}'").into()
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_version, resolve_version};

    #[test]
    fn parse_version_accepts_three_part_release_tags() {
        assert_eq!(parse_version("v3.13.1"), Some([3, 13, 1]));
        assert_eq!(parse_version("3.9.20"), Some([3, 9, 20]));
    }

    #[test]
    fn parse_version_rejects_incomplete_or_non_numeric_tags() {
        // A short tag must not panic via indexing (finding #6).
        assert_eq!(parse_version("v3"), None);
        assert_eq!(parse_version("v3.13"), None);
        assert_eq!(parse_version("v3.13.0rc1"), None);
        assert_eq!(parse_version(""), None);
    }

    #[tokio::test]
    async fn resolve_version_passes_through_full_versions_without_network() {
        assert_eq!(resolve_version("3.13.1").await.unwrap(), "v3.13.1");
        assert_eq!(resolve_version("v3.13.1").await.unwrap(), "v3.13.1");
    }

    #[tokio::test]
    async fn resolve_version_rejects_empty_and_overlong_input() {
        assert!(resolve_version("").await.is_err());
        assert!(resolve_version("3.13.1.2").await.is_err());
    }
}
