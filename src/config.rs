use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::path::Path;
use std::path::PathBuf;

/// All the configuration options for the vault wrapped in a single struct
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Config {
    pub general: General,
    pub appearance: Appearance,
    pub links: Vec<Link>,
    pub languages: Vec<Language>,
}

impl Config {
    /// Read a config file from disk and parse it
    pub fn from_disk<P>(path: P) -> Result<Config>
    where
        P: AsRef<Path>,
    {
        let file =
            std::fs::read_to_string(path).with_context(|| "Failed to read the config file.")?;

        let config: Config =
            serde_yaml::from_str(&file).with_context(|| "Failed to parse the config file")?;

        Ok(config)
    }

    /// Update the config with the values from another config
    pub fn update(&mut self, other: Config) {
        self.general = other.general;
        self.appearance = other.appearance;
        self.links = other.links;
        self.languages = other.languages;
    }

    /// Saves the config in the given path
    pub fn save<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let serialized = serde_yaml::to_string(self)
            .with_context(|| anyhow!("Failed to parse the config file"))?;

        std::fs::write(path, serialized)
            .with_context(|| anyhow!("Failed to write the config file"))?;

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Config {
        let general = General {
            title: String::new(),
            authors: vec![String::new()],
            desc: String::new(),
            enumerate: false,
            ignore: vec![],
            multiple_language: false,
            src_dir: PathBuf::from("src"),
            build_dir: PathBuf::from("build"),
            use_default: true,
        };

        let appearance = Appearance {
            custom: vec![],
            default_theme: String::from("gruvbox"),
            themes: vec!["gruvbox".to_string(), "catppuccin".to_string()],
        };

        let config = Config {
            general,
            appearance,
            links: vec![],
            languages: vec![],
        };

        config
    }
}

/// General configuration options for the vault
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct General {
    /// Title for the vault
    pub title: String,
    /// Authors of the vault
    pub authors: Vec<String>,
    /// A description for the vault
    pub desc: String,
    /// Should the chapters be enumerated?
    pub enumerate: bool,
    /// Files that should be ignored (e.g. not_ready.md)
    pub ignore: Vec<String>,
    /// Should multiple languages be available?
    pub multiple_language: bool,
    /// Should default css and js be used?
    pub use_default: bool,
    pub build_dir: PathBuf,
    pub src_dir: PathBuf,
}

/// Appearance options for the generated site
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Appearance {
    /// Paths to custom CSS files
    pub custom: Vec<String>,
    /// The theme that should be used by default
    pub default_theme: String,
    /// All available themes
    pub themes: Vec<String>,
}

/// Holds a link that should be displayed in the header
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Link {
    /// The link's name. If there is no icon available, this will be displayed as label
    pub name: String,
    /// The link's url
    pub url: String,
    /// An optional icon that should be displayed instead of the name
    pub icon: Option<String>,
}

/// Holds a Language e.g translation. It will be available in the header if multiple_language is true
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Language {
    /// The language's name (e.g. English, pt-br, etc.)
    pub name: String,
    /// Path to a directory containing the translated markdown files
    pub path: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use tempfile::tempdir;

    #[test]
    fn it_should_write_a_config_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("test_config.yml");
        let config = Config::default();

        config.save(&config_path)?;

        assert!(config_path.exists());

        Ok(())
    }

    #[test]
    fn it_should_read_a_config_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("test_config.yml");
        let config = Config::default();

        config.save(&config_path)?;

        let read_config = Config::from_disk(&config_path)?;

        assert_eq!(config, read_config);

        Ok(())
    }

    #[test]
    fn it_should_write_and_read_a_config_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("test_config.yml");
        let config = Config::default();

        config.save(&config_path)?;

        let read_config = Config::from_disk(&config_path)?;

        assert_eq!(config, read_config);

        Ok(())
    }

    #[test]
    fn it_should_update_a_config() -> Result<()> {
        let mut config = Config::default();
        let mut other = Config::default();

        other.general.title = "New Title".to_string();

        config.update(other);

        assert_eq!(config.general.title, "New Title");

        Ok(())
    }
}
