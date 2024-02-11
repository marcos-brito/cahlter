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

pub fn copy_dir<P>(source: P, destination: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let source = source.as_ref();
    let destination = destination.as_ref();

    for entry in source.read_dir()? {
        let entry = entry?;

        if entry.file_type()?.is_dir() {
            fs::create_dir(destination.join(entry.file_name()))?;
            copy_dir(entry.path(), destination.join(entry.file_name()))?
        } else {
            fs::copy(entry.path(), destination.join(entry.file_name()))?;
        }
    }

    Ok(())
}

pub fn remove_whitespace<S>(s: S) -> String
where
    S: AsRef<str>,
{
    s.as_ref().chars().filter(|c| !c.is_whitespace()).collect()
}
