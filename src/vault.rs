pub mod content;

use crate::config::Config;
use crate::renderer::{self, AskamaRenderer, Renderer};
use crate::util;
use crate::Chapter;
use anyhow::{Context, Result};
use content::Content;
use serde_yaml;
use std::fs;
use std::path::{Path, PathBuf};

pub const STYLES_DIR: &str = "styles";
pub const CONFIG_FILE: &str = "cahlter.yml";

pub struct Vault {
    pub config: Config,
    pub path: PathBuf,
}

impl Vault {
    pub fn new<P>(path: P) -> Vault
    where
        P: AsRef<Path>,
    {
        let config = Config::default();

        Vault {
            config,
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn from_disk<P>(path: P) -> Vault
    where
        P: AsRef<Path>,
    {
        let config = Config::from_disk(path.as_ref().join(CONFIG_FILE));

        Vault {
            config,
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Initialize a new vault at the given path. It also updates the config
    /// so the title is the name of the directory.
    pub fn init(&mut self) -> Result<()> {
        let mut new_config = Config::default();

        if Vault::was_initialized(&self.path) {
            anyhow::bail!("calhter.yml already exists at {}", self.path.display());
        }

        self.create()?;
        new_config.general.title = self.path.file_name().unwrap().to_string_lossy().to_string();
        self.config.update(new_config);
        self.config.save(self.path.join(CONFIG_FILE));

        Ok(())
    }

    fn create(&self) -> Result<()> {
        let dirs = vec![self.path.clone(), self.src_dir(), self.build_dir()];

        for dir in dirs {
            fs::create_dir_all(&dir)
                .with_context(|| format!("Could not create {}", dir.display()))?;
        }

        fs::write(
            self.path.join(CONFIG_FILE),
            serde_yaml::to_string(&Config::default())?,
        )
        .with_context(|| format!("Could not write {}.", self.path.join(CONFIG_FILE).display()))?;

        Ok(())
    }

    pub fn build(&mut self) -> Result<()> {
        let content = Content::new(self.src_dir())?;
        let context = renderer::Context::new(content.clone(), self.config.clone());
        let renderer = AskamaRenderer::new(context);
        let chapters = content.chapters();

        for chapter in chapters.iter() {
            self.write_chapter(&chapter, renderer.clone(), self.build_dir())?;
        }

        util::copy_dir(self.path.join(STYLES_DIR), self.build_dir())?;

        Ok(())
    }

    fn write_chapter<R, P>(&self, chapter: &Chapter, renderer: R, destination: P) -> Result<()>
    where
        R: Renderer + Clone,
        P: AsRef<Path>,
    {
        let file_name = chapter
            .content
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            + ".html";

        match chapter.subchapters.is_empty() {
            true => {
                let renderered = renderer.render(chapter).with_context(|| {
                    format!("Could not render chapter at {:?}", chapter.content)
                })?;

                let _ = fs::write(destination.as_ref().join(file_name), renderered);
            }
            false => {
                let destination = self
                    .build_dir()
                    .join(chapter.content.parent().unwrap().file_stem().unwrap());

                fs::create_dir(&destination)?;

                let renderered = renderer.render(chapter).with_context(|| {
                    format!("Could not render chapter at {:?}", chapter.content)
                })?;

                let _ = fs::write(destination.join(file_name), renderered);

                for subchapter in chapter.subchapters.iter() {
                    self.write_chapter(&subchapter, renderer.clone(), &destination)?;
                }
            }
        }

        Ok(())
    }

    fn was_initialized<P>(path: P) -> bool
    where
        P: AsRef<Path>,
    {
        path.as_ref().to_path_buf().join(CONFIG_FILE).exists()
    }

    pub fn src_dir(&self) -> PathBuf {
        self.path.join(&self.config.general.src_dir)
    }

    pub fn build_dir(&self) -> PathBuf {
        self.path.join(&self.config.general.build_dir)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::error::Error;
    use tempfile::tempdir;

    #[test]
    fn it_should_initialize_the_vault() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let mut vault = Vault::new(temp_dir.path().join("test_vault"));

        vault.init()?;

        assert!(vault.path.exists());
        assert!(vault.config.general.title == "test_vault");

        Ok(())
    }

    #[test]
    #[should_panic(expected = "calhter.yml already exists at")]
    fn it_should_not_initialize_the_vault_if_it_was_already_initialized() -> () {
        let temp_dir = tempdir().unwrap();
        let mut vault = Vault::new(temp_dir.path().join("test_vault"));

        vault.init().unwrap();
        vault.init().unwrap();
    }

    #[test]
    fn it_should_create_the_right_directories() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let vault = Vault::new(temp_dir.path().join("test_vault"));

        vault.create()?;

        assert!(vault.src_dir().exists());
        assert!(vault.build_dir().exists());
        assert!(temp_dir
            .path()
            .join("test_vault")
            .join(CONFIG_FILE)
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

        vault.init()?;

        assert!(Vault::was_initialized(temp_dir.path().join("test_vault")));

        Ok(())
    }

    #[test]
    fn it_should_build_the_vault() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let mut vault = Vault::new(temp_dir.path());
        vault.init()?;

        let _ = fs::write(vault.src_dir().join("chapter1.md"), "# Hello there");
        let _ = fs::write(
            vault.src_dir().join("chapter2.md"),
            "> Here is where the fun begins",
        );
        vault.build()?;

        assert!(vault.src_dir().join("chapter1.md").exists());
        assert!(vault.src_dir().join("chapter2.md").exists());
        assert!(temp_dir.path().join(STYLES_DIR).join("main.css").exists());

        Ok(())
    }
}
