use homedir::get_my_home;
use std::error::Error;
use std::path::PathBuf;

/// Resolve the `~/.vmp` base directory, returning a descriptive error instead of
/// panicking when the current user's home directory cannot be determined.
pub fn vmp_dir() -> Result<PathBuf, Box<dyn Error>> {
    let home = get_my_home()
        .map_err(|error| format!("failed to resolve home directory: {error}"))?
        .ok_or("could not determine the current user's home directory")?;
    Ok(home.join(".vmp"))
}
