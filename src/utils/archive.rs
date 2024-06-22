use std::fs::File;

use flate2::read::GzDecoder;
use tar::Archive;

pub fn extract_tar_gz(source: String, dest: &String, prefix: String) -> Result<(), std::io::Error> {
    let tar_gz = File::open(source)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        let stripped_path = path.strip_prefix(&prefix).unwrap().to_str().unwrap();
        let dst = format!("{}/{}", dest, stripped_path);

        println!("{}", dst);
        _ = entry.unpack(dst);
    }

    Ok(())
}
