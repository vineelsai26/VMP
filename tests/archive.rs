use flate2::{write::GzEncoder, Compression};
use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tar::{Builder, Header};
use vmp::utils::archive::extract_tar_gz;

struct TestDir(PathBuf);

impl TestDir {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("vmp-{name}-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&path).expect("create test directory");
        Self(path)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_archive(path: &Path, entries: &[(&str, &[u8])]) -> io::Result<()> {
    let encoder = GzEncoder::new(File::create(path)?, Compression::default());
    let mut archive = Builder::new(encoder);

    for (entry_path, contents) in entries {
        let mut header = Header::new_gnu();
        header.set_path(entry_path)?;
        header.set_size(contents.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive.append(&header, *contents)?;
    }

    archive.into_inner()?.finish()?;
    Ok(())
}

fn write_archive_with_raw_path(path: &Path, entry_path: &[u8], contents: &[u8]) -> io::Result<()> {
    let encoder = GzEncoder::new(File::create(path)?, Compression::default());
    let mut archive = Builder::new(encoder);
    let mut header = Header::new_gnu();
    header.as_old_mut().name[..entry_path.len()].copy_from_slice(entry_path);
    header.set_size(contents.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    archive.append(&header, contents)?;
    archive.into_inner()?.finish()?;
    Ok(())
}

#[test]
fn extracts_entries_under_the_expected_prefix() {
    let root = TestDir::new("archive-valid");
    let archive = root.0.join("Python-3.12.0.tgz");
    let destination = root.0.join("output");
    write_archive(
        &archive,
        &[("Python-3.12.0/Lib/example.py", b"print('ok')\n")],
    )
    .expect("write archive");

    extract_tar_gz(
        archive.to_string_lossy().into_owned(),
        &destination.to_string_lossy().into_owned(),
        "Python-3.12.0".to_string(),
    )
    .expect("extract valid archive");

    assert_eq!(
        fs::read(destination.join("Lib/example.py")).expect("read extracted file"),
        b"print('ok')\n"
    );
}

#[test]
fn rejects_entries_outside_the_expected_prefix_without_panicking() {
    let root = TestDir::new("archive-prefix");
    let archive = root.0.join("Python-3.12.0.tgz");
    let destination = root.0.join("output");
    write_archive(&archive, &[("unexpected/file", b"malicious")]).expect("write archive");

    let result = extract_tar_gz(
        archive.to_string_lossy().into_owned(),
        &destination.to_string_lossy().into_owned(),
        "Python-3.12.0".to_string(),
    );

    assert!(result.is_err());
    assert!(!destination.join("file").exists());
}

#[test]
fn rejects_parent_directory_traversal() {
    let root = TestDir::new("archive-traversal");
    let archive = root.0.join("Python-3.12.0.tgz");
    let destination = root.0.join("nested/output");
    let escaped = root.0.join("escaped");
    write_archive_with_raw_path(&archive, b"Python-3.12.0/../../escaped", b"malicious")
        .expect("write archive");

    let result = extract_tar_gz(
        archive.to_string_lossy().into_owned(),
        &destination.to_string_lossy().into_owned(),
        "Python-3.12.0".to_string(),
    );

    assert!(result.is_err());
    assert!(!escaped.exists());
}

#[test]
fn propagates_destination_write_failures() {
    let root = TestDir::new("archive-write-error");
    let archive = root.0.join("Python-3.12.0.tgz");
    let destination = root.0.join("output");
    write_archive(&archive, &[("Python-3.12.0/README", b"contents")]).expect("write archive");
    fs::write(&destination, b"not a directory").expect("create destination collision");

    let result = extract_tar_gz(
        archive.to_string_lossy().into_owned(),
        &destination.to_string_lossy().into_owned(),
        "Python-3.12.0".to_string(),
    );

    assert!(result.is_err());
}
