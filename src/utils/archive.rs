use flate2::read::GzDecoder;
use std::{
    fs::{self, File},
    io,
    path::{Component, Path, PathBuf},
};
use tar::Archive;

pub fn extract_tar_gz(source: String, dest: &String, prefix: String) -> io::Result<()> {
    let tar_gz = File::open(source)?;
    extract_tar_gz_from_reader(tar_gz, dest, prefix)
}

/// Extract a gzip-compressed tar archive from an already-open reader.
///
/// Callers that must not re-read the archive from disk between verification and
/// extraction (to avoid a TOCTOU swap) pass the verified bytes in directly,
/// e.g. via `io::Cursor`.
pub fn extract_tar_gz_from_reader<R: io::Read>(
    reader: R,
    dest: &String,
    prefix: String,
) -> io::Result<()> {
    let destination = Path::new(dest);
    let prefix = Path::new(&prefix);
    validate_prefix(prefix)?;

    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let staging = create_staging_dir(parent)?;

    let result = (|| {
        let mut archive = Archive::new(GzDecoder::new(reader));

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.into_owned();
            path.strip_prefix(prefix).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "archive entry '{}' is outside expected prefix '{}'",
                        path.display(),
                        prefix.display()
                    ),
                )
            })?;

            if !entry.unpack_in(&staging)? {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unsafe archive entry '{}'", path.display()),
                ));
            }
        }

        let extracted_root = staging.join(prefix);
        if !extracted_root.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("archive is missing expected prefix '{}'", prefix.display()),
            ));
        }

        if let Ok(metadata) = fs::symlink_metadata(destination) {
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "extraction destination '{}' is not a directory",
                        destination.display()
                    ),
                ));
            }
            fs::remove_dir_all(destination)?;
        }

        fs::rename(extracted_root, destination)
    })();

    let cleanup_result = fs::remove_dir_all(&staging);
    match result {
        Ok(()) => cleanup_result,
        Err(error) => {
            let _ = cleanup_result;
            Err(error)
        }
    }
}

fn validate_prefix(prefix: &Path) -> io::Result<()> {
    if prefix.as_os_str().is_empty()
        || prefix
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid archive prefix '{}'", prefix.display()),
        ));
    }
    Ok(())
}

fn create_staging_dir(parent: &Path) -> io::Result<PathBuf> {
    for attempt in 0..100 {
        let path = parent.join(format!(".vmp-extract-{}-{attempt}", std::process::id()));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not allocate a temporary extraction directory",
    ))
}
