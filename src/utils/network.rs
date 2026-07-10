use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::{cmp::min, fs::File, io::Write};

pub async fn download_file(
    url: String,
    file_path: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let res = reqwest::get(&url).await?.error_for_status()?;

    let download_size = res.content_length().unwrap_or(0);

    let pb = ProgressBar::new(download_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
            .progress_chars("#>-")
    );
    pb.set_message("Downloading ".to_string() + &url);

    let mut file =
        File::create(file_path).or(Err(format!("Failed to create file '{}'", file_path)))?;
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

    Ok(())
}
