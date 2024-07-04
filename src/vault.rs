pub mod content;

use crate::config::Config;
use crate::renderer::{self, AskamaRenderer, Renderer};
use crate::Chapter;
use anyhow::{anyhow, Context, Result};
use content::Content;
use serde_yaml;
use std::fs;
use std::path::{Path, PathBuf};

static CSS: &[u8] = include_bytes!("../templates/main.css");
static JS: &[u8] = include_bytes!("../templates/index.js");
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

    pub fn from_disk<P>(path: P) -> Result<Vault>
    where
        P: AsRef<Path>,
    {
        let config = Config::from_disk(path.as_ref().join(CONFIG_FILE))?;

        Ok(Vault {
            config,
            path: path.as_ref().to_path_buf(),
        })
    }

    /// Initialize a new vault at the given path. It also updates the config
    /// so the title is the name of the directory.
    pub fn init(&mut self) -> Result<()> {
        if Vault::was_initialized(&self.path) {
            anyhow::bail!("calhter.yml already exists at {}", self.path.display());
        }

        self.create()?;
        self.config.general.title = self.path.file_name().unwrap().to_string_lossy().to_string();
        self.config.save(self.path.join(CONFIG_FILE))?;

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
            serde_yaml::to_string(&self.config)?,
        )
        .with_context(|| format!("Could not write {}.", self.path.join(CONFIG_FILE).display()))?;

        Ok(())
    }

    pub fn build(&mut self) -> Result<()> {
        let content = Content::new(self.src_dir())?;
        let context =
            renderer::RendererContext::new(content.clone(), self.config.clone(), self.src_dir());
        let renderer = AskamaRenderer::new(context);
        let chapters = content.chapters();

        for chapter in chapters.iter() {
            self.write_chapter(&chapter, renderer.clone(), self.build_dir())?;
        }

        if self.config.general.use_default {
            for static_file in vec![CSS, JS] {
                fs::write(self.build_dir().join("main.css"), static_file)
                    .with_context(|| anyhow!("Failed to write default files"))?;
            }
        }

        for css_file in self.config.appearance.custom.iter() {
            let file_name = Path::new(css_file).file_name().unwrap();

            fs::copy(self.path.join(css_file), self.build_dir().join(file_name))?;
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
                fs::write(
                    destination.as_ref().join(&file_name),
                    renderer.render(chapter)?,
                )?;
            }
            false => {
                let destination = self
                    .build_dir()
                    .join(chapter.content.parent().unwrap().file_stem().unwrap());
                fs::create_dir(&destination)?;

                fs::write(destination.join(&file_name), renderer.render(chapter)?)?;

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

        fs::write(vault.src_dir().join("chapter1.md"), "# Hello there")?;
        fs::write(
            vault.src_dir().join("chapter2.md"),
            "> Here is where the fun begins",
        )?;
        vault.build()?;

        assert!(vault.build_dir().join("chapter1.html").exists());
        assert!(vault.build_dir().join("chapter2.html").exists());
        assert!(vault.build_dir().join("main.css").exists());

        Ok(())
    }

    #[test]
    fn it_should_build_the_vault_with_custom_css() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let mut vault = Vault::new(temp_dir.path());

        vault.config.general.use_default = false;
        vault.config.appearance.custom = vec![
            "./custom1.css".to_string(),
            "./css/custom2.css".to_string(),
            "./css/styles/custom3.css".to_string(),
        ];
        vault.init()?;

        fs::create_dir_all(vault.path.join("css").join("styles"))?;
        fs::write(vault.src_dir().join("chapter1.md"), "# Hello there")?;
        fs::write(vault.path.join("custom1.css"), "")?;
        fs::write(vault.path.join("css/custom2.css"), "")?;
        fs::write(vault.path.join("css/styles/custom3.css"), "")?;

        vault.build()?;

        assert!(vault.build_dir().join("chapter1.html").exists());
        assert!(!vault.build_dir().join("main.css").exists());
        assert!(vault.build_dir().join("custom1.css").exists());
        assert!(vault.build_dir().join("custom2.css").exists());
        assert!(vault.build_dir().join("custom3.css").exists());

        Ok(())
    }
}
