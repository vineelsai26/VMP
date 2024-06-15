use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::{cmp::min, fs::File, io::Write};

pub async fn download_file(
    url: String,
    file_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let res = reqwest::get(url.clone()).await?;

    let download_size = res.content_length().expect("");

    let pb = ProgressBar::new(download_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
            .progress_chars("#>-")
    );
    pb.set_message("Downloading ".to_string() + &url);

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
