pub mod content;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_yaml;

use crate::config::Config;
use crate::renderer::{self, AskamaRenderer, Renderer};
use crate::{util, Chapter};
use content::Content;

pub const BUILD_DIR: &str = "build";
pub const SRC_DIR: &str = "src";
pub const STYLES_DIR: &str = "styles";
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

    pub fn from_disk<P>(path: P) -> Vault
    where
        P: AsRef<Path>,
    {
        let config = Config::from_disk(&path);
        let content = Vault::get_content(&path);

        Vault {
            content,
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
        fs::create_dir_all(&self.path).with_context(|| {
            format!(
                "Could not create {}. Maybe it's invalid or you don't have permission",
                &self.path.display()
            )
        })?;

        util::create_dir_if_not_exists(self.path.join(BUILD_DIR)).with_context(|| {
            format!("Could not create {}.", self.path.join(BUILD_DIR).display())
        })?;

        util::create_dir_if_not_exists(self.path.join("styles"))
            .with_context(|| format!("Could not create {}.", self.path.join("styles").display()))?;

        util::create_dir_if_not_exists(self.path.join(SRC_DIR))
            .with_context(|| format!("Could not create {}.", self.path.join(SRC_DIR).display()))?;

        fs::write(
            self.path.join(CONFIG_FILE),
            serde_yaml::to_string(&Config::default())?,
        )
        .with_context(|| {
            format!(
                "Could not create {}.",
                self.path.join(CONFIG_FILE).display()
            )
        })?;

        Ok(())
    }

    pub fn build(&mut self) -> Result<()> {
        // make sure we got the most recent content
        self.content = Some(Content::new(&self.path.join(SRC_DIR)));

        let context = renderer::Context::new(self.content.clone().unwrap(), self.config.clone());
        let renderer = AskamaRenderer::new(context);
        let chapters = self.content.clone().unwrap().chapters();

        // why the fuck it can't take a PathBuf? The docs says it implements AsRef<Path> 😿
        util::copy_dir("templates/css", self.path.join("styles").to_str().unwrap())
            .with_context(|| "Could not move style files")?;

        for chapter in chapters.iter() {
            self.write_chapter(&chapter, renderer.clone(), self.path.join(BUILD_DIR))?;
        }

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
                    .path
                    .join(BUILD_DIR)
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
    use tempfile::tempdir;

    #[test]
    fn it_should_initialize_the_vault() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let mut vault = Vault::new(temp_dir.path().join("test_vault"));

        vault.init()?;

        assert!(temp_dir.path().join("test_vault").exists());
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

        assert!(temp_dir.path().join("test_vault").join("build").exists());
        assert!(temp_dir.path().join("test_vault").join("styles").exists());
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

        vault.init()?;

        assert!(Vault::was_initialized(temp_dir.path().join("test_vault")));

        Ok(())
    }

    #[test]
    fn it_should_build_the_vault() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let mut vault = Vault::new(temp_dir.path());
        vault.init()?;

        let _ = fs::write(
            temp_dir.path().join(SRC_DIR).join("chapter1.md"),
            "# Hello there",
        );
        let _ = fs::write(
            temp_dir.path().join(SRC_DIR).join("chapter2.md"),
            "> Here is where the fun begins",
        );
        vault.build()?;

        assert!(temp_dir.path().join(SRC_DIR).join("chapter1.md").exists());
        assert!(temp_dir.path().join(SRC_DIR).join("chapter2.md").exists());
        assert!(temp_dir.path().join("styles").join("main.css").exists());

        Ok(())
    }
}
