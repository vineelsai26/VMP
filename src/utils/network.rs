use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{redirect::Policy, Client, Url};
use std::{cmp::min, fs::File, io::Write, path::Path};

pub async fn download_file(
    url: String,
    file_path: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let parsed = Url::parse(&url)?;
    if parsed.scheme() != "https" || parsed.host_str().is_none() {
        return Err(format!("download URL must use HTTPS: {url}").into());
    }
    let expected_origin = parsed.origin().ascii_serialization();
    let client = Client::builder()
        .redirect(Policy::custom(move |attempt| {
            if attempt.previous().len() >= 5 {
                return attempt.error("too many redirects");
            }
            if attempt.url().scheme() != "https"
                || attempt.url().origin().ascii_serialization() != expected_origin
            {
                return attempt.error("refusing cross-origin or non-HTTPS redirect");
            }
            attempt.follow()
        }))
        .build()?;
    let res = client.get(parsed).send().await?.error_for_status()?;

    let download_size = res.content_length().unwrap_or(0);

    let pb = ProgressBar::new(download_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
            .progress_chars("#>-")
    );
    pb.set_message("Downloading ".to_string() + &url);

    let destination = Path::new(file_path);
    let temporary = destination.with_extension(format!(
        "{}.part",
        destination
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("download")
    ));
    let mut file = File::create(&temporary).map_err(|error| {
        format!(
            "failed to create temporary download '{}': {error}",
            temporary.display()
        )
    })?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|error| format!("error while downloading file: {error}"))?;
        file.write_all(&chunk)
            .map_err(|error| format!("error while writing to file: {error}"))?;
        let new = min(downloaded + (chunk.len() as u64), download_size);
        downloaded = new;
        pb.set_position(new);
    }
    file.sync_all()?;
    drop(file);
    std::fs::rename(&temporary, destination)?;

    Ok(())
}
