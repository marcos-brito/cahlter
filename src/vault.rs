pub mod content;
pub mod summary;

use super::config::Config;
use content::Content;
use serde_yaml;
use std::fs;
use std::path::{Path, PathBuf};

pub const BUILD_DIR: &str = "build";
pub const SRC_DIR: &str = "src";
pub const CONFIG_FILE: &str = "cahlter.yml";

pub struct Vault {
    pub content: Option<Content>,
    pub config: Config,
    pub path: PathBuf,
}

impl Vault {
    pub fn new<P>(path: P) -> Vault
    where
        P: AsRef<Path>,
    {
        let config = Config::default();
        let content = Vault::get_content(&path);

        Vault {
            content,
            config,
            path: path.as_ref().to_path_buf(),
        }
    }
    /// Initialize a new vault at the given path. It also updates the config
    /// so that the title is the name of the directory.
    pub fn init(&mut self) -> () {
        let mut new_config = Config::default();

        self.create();
        new_config.general.title = self.path.file_name().unwrap().to_string_lossy().to_string();
        self.config.update(new_config);
    }

    fn create(&self) -> () {
        let error_msg = format!(
            "Could not create directory. Maybe you don't have permission? Maybe a invalid path? {}",
            self.path.display()
        );

        fs::create_dir_all(&self.path).expect(&error_msg);
        fs::create_dir(self.path.join(BUILD_DIR)).expect(&error_msg);
        fs::create_dir(self.path.join(SRC_DIR)).expect(&error_msg);
        fs::write(
            self.path.join(CONFIG_FILE),
            serde_yaml::to_string(&Config::default()).unwrap(),
        )
        .expect(&error_msg);
    }

    fn get_content<P>(path: P) -> Option<Content>
    where
        P: AsRef<Path>,
    {
        if Vault::was_initialized(&path) {
            Some(Content::new(path.as_ref().to_path_buf().join(SRC_DIR)))
        } else {
            None
        }
    }

    fn was_initialized<P>(path: P) -> bool
    where
        P: AsRef<Path>,
    {
        path.as_ref().to_path_buf().join(CONFIG_FILE).exists()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::error::Error;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn it_should_initialize_the_vault() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let mut vault = Vault::new(temp_dir.path().join("test_vault"));

        vault.init();

        assert!(temp_dir.path().join("test_vault").exists());
        assert!(vault.config.general.title == "test_vault");

        Ok(())
    }

    #[test]
    fn it_should_create_the_right_directories() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let vault = Vault::new(temp_dir.path().join("test_vault"));

        vault.create();

        assert!(temp_dir.path().join("test_vault").join("build").exists());
        assert!(temp_dir.path().join("test_vault").join("src").exists());
        assert!(temp_dir
            .path()
            .join("test_vault")
            .join("cahlter.yml")
            .exists());

        Ok(())
    }

    #[test]
    fn it_should_return_false_if_the_vault_was_not_initialized() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;

        assert!(!Vault::was_initialized(temp_dir.path().join("test_vault")));

        Ok(())
    }

    #[test]
    fn it_should_return_true_if_the_vault_was_initialized() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let mut vault = Vault::new(temp_dir.path().join("test_vault"));

        vault.init();

        assert!(Vault::was_initialized(temp_dir.path().join("test_vault")));

        Ok(())
    }
}