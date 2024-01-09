use serde::{Deserialize, Serialize};
use serde_yaml;
use std::path::Path;

/// All the configuration options for the vault wrapped in a single struct
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    pub general: General,
    pub appearance: Appearance,
    pub links: Option<Vec<Link>>,
    pub languages: Option<Vec<Language>>,
}

impl Config {
    /// Read a config file from disk and parse it
    pub fn from_disk<P>(path: P) -> Config
    where
        P: AsRef<Path>,
    {
        let file =
            std::fs::read_to_string(path).expect("Could not read config file. Does it exist?");

        let config: Config = serde_yaml::from_str(&file)
            .expect("Could not parse config file. Maybe it is not valid YAML?");

        config
    }

    /// Update the config with the values from another config
    pub fn update(&mut self, other: Config) {
        self.general = other.general;
        self.appearance = other.appearance;
        self.links = other.links;
        self.languages = other.languages;
    }

    /// Saves the config in the given path
    pub fn save<P>(&self, path: P) -> ()
    where
        P: AsRef<Path>,
    {
        let serialized = serde_yaml::to_string(self).unwrap();
        std::fs::write(path, serialized).expect("Could not write config file");
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
        };

        let appearance = Appearance {
            custom: vec![],
            default_theme: String::new(),
            themes: vec![],
        };

        let config = Config {
            general,
            appearance,
            links: None,
            languages: None,
        };

        config
    }
}

/// General configuration options for the vault
#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
}

/// Appearance options for the generated site
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Appearance {
    /// Paths to custom CSS files
    pub custom: Vec<String>,
    /// The theme that should be used by default
    pub default_theme: String,
    /// All available themes
    pub themes: Vec<Theme>,
}

/// Holds information about a theme
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Theme {
    /// The theme's name
    pub name: String,
    /// Path to a CSS file
    pub path: String,
}

/// Holds a link that should be displayed in the header
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Link {
    /// The link's name. If there is no icon available, this will be displayed as label
    pub name: String,
    /// The link's url
    pub url: String,
    /// An optional icon that should be displayed instead of the name
    pub icon: Option<String>,
}

/// Holds a Language e.g translation. It will be available in the header if multiple_language is true
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Language {
    /// The language's name (e.g. English, pt-br, etc.)
    pub name: String,
    /// Path to a directory containing the translated markdown files
    pub path: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{error::Error, fs};
    use tempfile::tempdir;

    #[test]
    fn it_should_write_a_config_file() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("test_config.yml");
        let config = Config::default();

        config.save(&config_path);

        assert!(config_path.exists());

        Ok(())
    }

    #[test]
    fn it_should_read_a_config_file() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("test_config.yml");
        let config = Config::default();

        config.save(&config_path);

        let read_config = Config::from_disk(&config_path);

        assert_eq!(config, read_config);

        Ok(())
    }

    #[test]
    fn it_should_write_and_read_a_config_file() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("test_config.yml");
        let config = Config::default();

        config.save(&config_path);

        let read_config = Config::from_disk(&config_path);

        assert_eq!(config, read_config);

        Ok(())
    }

    #[test]
    #[should_panic(expected = "Could not read config file. Does it exist?")]
    fn it_should_panic_when_file_does_not_exist() {
        let config_path = "non_existent_file.yml";

        Config::from_disk(&config_path);
    }

    #[test]
    #[should_panic(expected = "Could not parse config file. Maybe it is not valid YAML?")]
    fn it_should_panic_if_config_is_not_valid_yaml() -> () {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.yml");
        let config = r#"name: "John Doe" 
        age: 25
        city: "New York"
        hasJob: true
        hobbies:
        - Reading
        - Swimming"#;

        fs::write(&config_path, config).unwrap();

        Config::from_disk(&config_path);
    }

    #[test]
    fn it_should_update_a_config() -> Result<(), Box<dyn Error>> {
        let mut config = Config::default();
        let mut other = Config::default();

        other.general.title = "New Title".to_string();

        config.update(other);

        assert_eq!(config.general.title, "New Title");

        Ok(())
    }
}
