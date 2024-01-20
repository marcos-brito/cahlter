use super::{Context, Renderer};
use crate::config::Link;

use anyhow::Result;
use askama::Template;

#[derive(Template)]
#[template(path = "header.html")]
struct Header<'a> {
    links: &'a Vec<Link>,
}

pub struct AskamaRenderer<'a> {
    pub context: &'a Context,
}

impl<'a> AskamaRenderer<'a> {
    pub fn new(context: &'a Context) -> Self {
        Self { context }
    }

    pub fn render_header(&self) -> Result<String> {
        let links = self.context.config.links.clone().unwrap_or(Vec::new());
        let header = Header { links: &links };

        Ok(header.render()?)
    }
}

impl Renderer for AskamaRenderer<'_> {
    fn render(&self) -> Result<String> {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::Config;
    use crate::Content;
    use std::error::Error;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn it_should_render_the_header() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let content = Content::new(temp_dir.path().to_path_buf());
        let links = vec![
            Link {
                name: "GitHub".to_string(),
                url: "https://github.com".to_string(),
                icon: None,
            },
            Link {
                name: "Twitter".to_string(),
                url: "https://twitter.com".to_string(),
                icon: None,
            },
        ];

        let mut config = Config::default();
        config.general.title = "Test".to_string();
        config.links = Some(links);

        let context = Context::new(content, config);
        let renderer = AskamaRenderer::new(&context);

        let output = renderer
            .render_header()?
            .chars()
            .filter_map(|c| if c.is_whitespace() { None } else { Some(c) })
            .collect::<String>();
        let expected = fs::read_to_string("tests/testdata/header.html")?
            .chars()
            .filter_map(|c| if c.is_whitespace() { None } else { Some(c) })
            .collect::<String>();

        assert_eq!(output, expected);

        Ok(())
    }

    #[test]
    fn it_should_render_the_header_if_links_are_empty() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let content = Content::new(temp_dir.path().to_path_buf());

        let mut config = Config::default();
        config.general.title = "Test".to_string();

        let context = Context::new(content, config);
        let renderer = AskamaRenderer::new(&context);

        let output = renderer
            .render_header()?
            .chars()
            .filter_map(|c| if c.is_whitespace() { None } else { Some(c) })
            .collect::<String>();
        let expected = fs::read_to_string("tests/testdata/header_empty_links.html")?
            .chars()
            .filter_map(|c| if c.is_whitespace() { None } else { Some(c) })
            .collect::<String>();

        assert_eq!(output, expected);

        Ok(())
    }

    #[test]
    fn it_should_render_the_header_if_it_have_icons() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let content = Content::new(temp_dir.path().to_path_buf());
        let links = vec![
            Link {
                name: "GitHub".to_string(),
                url: "https://github.com".to_string(),
                icon: Some("github".to_string()),
            },
            Link {
                name: "Twitter".to_string(),
                url: "https://twitter.com".to_string(),
                icon: Some("twitter".to_string()),
            },
        ];

        let mut config = Config::default();
        config.general.title = "Test".to_string();
        config.links = Some(links);

        let context = Context::new(content, config);
        let renderer = AskamaRenderer::new(&context);

        let output = renderer
            .render_header()?
            .chars()
            .filter_map(|c| if c.is_whitespace() { None } else { Some(c) })
            .collect::<String>();
        let expected = fs::read_to_string("tests/testdata/header_with_icons.html")?
            .chars()
            .filter_map(|c| if c.is_whitespace() { None } else { Some(c) })
            .collect::<String>();

        assert_eq!(output, expected);

        Ok(())
    }
}
