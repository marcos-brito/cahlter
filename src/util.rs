use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn create_dir_if_not_exists<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref().to_path_buf();

    if !path.exists() {
        fs::create_dir_all(path)?;
    }

    Ok(())
}
