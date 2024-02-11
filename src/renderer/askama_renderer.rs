use anyhow::Result;
use askama::Template;

use super::{Context, Renderer};
use crate::config::Link;
use crate::{Chapter, Item, Section};

use std::fs;

#[derive(Template)]
#[template(path = "header.html")]
struct Header<'a> {
    links: &'a Vec<Link>,
}

#[derive(Template)]
#[template(path = "sidebar.html", escape = "none")]
struct Sidebar<'a> {
    title: &'a String,
    table_of_contents: &'a String,
}

#[derive(Template)]
#[template(path = "sidebar/chapter.html", escape = "none")]
struct SidebarChapter<'a> {
    title: &'a String,
    subchapters: &'a String,
    target: &'a String,
}

#[derive(Template)]
#[template(path = "sidebar/section.html")]
struct SidebarSection<'a> {
    title: &'a String,
}

#[derive(Template)]
#[template(path = "index.html", escape = "none")]
struct Page<'a> {
    header: &'a String,
    sidebar: &'a String,
    content: &'a String,
}

#[derive(Debug, Clone)]
pub struct AskamaRenderer {
    context: Context,
}

impl AskamaRenderer {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub fn render_header(&self) -> Result<String> {
        let links = self.context.config.links.clone().unwrap_or(Vec::new());
        let header = Header { links: &links };

        Ok(header.render()?)
    }

    // The template engine makes difficult to renderer the sidebar, so we do the heavy lifting here
    fn render_sidebar(&self) -> Result<String> {
        let title = self.context.config.general.title.clone();
        let items = self.context.content.summary.items.clone();
        let mut table_of_contents = String::new();

        for item in items.iter() {
            match item {
                Item::Chapter(chapter) => {
                    table_of_contents.push_str(&self.render_sidebar_chapter(&chapter)?)
                }
                Item::Section(section) => {
                    table_of_contents.push_str(&self.render_sidebar_section(&section)?)
                }
            }
        }

        let sidebar = Sidebar {
            title: &title,
            table_of_contents: &table_of_contents,
        };

        Ok(sidebar.render()?)
    }

    fn render_sidebar_chapter(&self, chapter: &Chapter) -> Result<String> {
        // We don't care about indentation here. The css class takes care of it.
        let subchapters = chapter
            .subchapters
            .iter()
            .map(|chapter| self.render_sidebar_chapter(&chapter))
            .collect::<Result<Vec<String>>>()?
            .join("");

        let target = chapter
            .content
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let title = match self.context.config.general.enumerate {
            true => format!("{} {}", chapter.number, chapter.title),
            false => chapter.title.clone(),
        };

        let sidebar_chapter = SidebarChapter {
            title: &title,
            subchapters: &subchapters,
            target: &(target + ".html"),
        };

        Ok(sidebar_chapter.render()?)
    }

    fn render_sidebar_section(&self, section: &Section) -> Result<String> {
        let sidebar_section = SidebarSection {
            title: &section.title,
        };

        Ok(sidebar_section.render()?)
    }
}

impl Renderer for AskamaRenderer {
    fn render(&self, chapter: &Chapter) -> Result<String> {
        let header = self.render_header()?;
        let sidebar = self.render_sidebar()?;

        let markdown = fs::read_to_string(&chapter.content).unwrap();
        let parser = pulldown_cmark::Parser::new(&markdown);
        let mut html = String::new();

        pulldown_cmark::html::push_html(&mut html, parser);

        let index = Page {
            header: &header,
            sidebar: &sidebar,
            content: &html,
        };

        return Ok(index.render()?);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::config::Config;
    use crate::util;
    use crate::Content;

    use std::error::Error;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn generate_renderer() -> AskamaRenderer {
        let temp_dir = tempdir().unwrap();
        let content = Content::new(temp_dir.path().to_path_buf());
        let config = Config::default();
        let context = Context::new(content, config);

        AskamaRenderer::new(context)
    }

    #[test]
    fn it_should_render_the_full_page() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        fs::create_dir(temp_dir.path().join("chapter1"))?;
        fs::write(
            temp_dir.path().join("chapter1/index.md"),
            "# This is a test",
        )?;
        fs::write(temp_dir.path().join("chapter1/subchapter1.md"), "")?;

        let mut renderer = generate_renderer();
        renderer.context.content = Content::new(&temp_dir);
        renderer.context.config.general.title = String::from("Test");

        for chapter in renderer.context.content.chapters().iter() {
            let expected_path =
                Path::new("tests/testdata").join(chapter.content.file_stem().unwrap());

            let output = util::remove_whitespace(renderer.render(&chapter)?);
            let expected = util::remove_whitespace(fs::read_to_string(&expected_path)?);

            assert_eq!(output, expected);
        }

        Ok(())
    }

    #[test]
    fn it_should_render_the_header() -> Result<(), Box<dyn Error>> {
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

        let mut renderer = generate_renderer();
        renderer.context.config.links = Some(links);

        let output = util::remove_whitespace(renderer.render_header()?);
        let expected = util::remove_whitespace(fs::read_to_string("tests/testdata/header.html")?);

        assert_eq!(output, expected);

        Ok(())
    }

    #[test]
    fn it_should_render_the_header_if_it_have_icons() -> Result<(), Box<dyn Error>> {
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

        let mut renderer = generate_renderer();
        renderer.context.config.links = Some(links);

        let output = util::remove_whitespace(renderer.render_header()?);
        let expected =
            util::remove_whitespace(fs::read_to_string("tests/testdata/header_with_icons.html")?);

        assert_eq!(output, expected);

        Ok(())
    }

    #[test]
    fn it_should_render_the_sidebar() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        fs::create_dir(temp_dir.path().join("chapter1"))?;
        fs::write(temp_dir.path().join("chapter1/subchapter1.1.md"), "")?;
        fs::write(temp_dir.path().join("chapter1/index.md"), "")?;
        fs::write(temp_dir.path().join("chapter1/subchapter1.2.md"), "")?;
        fs::write(temp_dir.path().join("chapter2.md"), "")?;

        let mut renderer = generate_renderer();
        let content = Content::new(temp_dir.path().to_path_buf());
        let mut config = Config::default();
        config.general.title = "Test".to_string();
        renderer.context = Context::new(content, config);

        let output = util::remove_whitespace(renderer.render_sidebar()?);
        let expected = util::remove_whitespace(fs::read_to_string("tests/testdata/sidebar.html")?);

        assert_eq!(output, expected);

        Ok(())
    }

    #[test]
    fn it_should_render_a_chapter_in_the_sidebar() -> Result<(), Box<dyn Error>> {
        let test_cases = vec![
            (
                Chapter::new("Chapter 1", "1", "chapter1.md", vec![]),
                "tests/testdata/sidebar/chapter.html",
            ),
            (
                Chapter::new(
                    "Chapter 1",
                    "1",
                    "chapter1.md",
                    vec![Chapter::new(
                        "Subchapter 1",
                        "1.1",
                        "subchapter1.md",
                        vec![],
                    )],
                ),
                "tests/testdata/sidebar/chapter_nested.html",
            ),
        ];

        let renderer = generate_renderer();

        for (chapter, expected_path) in test_cases {
            let output = util::remove_whitespace(renderer.render_sidebar_chapter(&chapter)?);
            let expected = util::remove_whitespace(fs::read_to_string(expected_path)?);

            assert_eq!(output, expected);
        }

        Ok(())
    }

    #[test]
    fn it_should_render_a_section_in_the_sidebar() -> Result<(), Box<dyn Error>> {
        let test_cases = vec![(
            Section::new("Section 1"),
            "tests/testdata/sidebar/section.html",
        )];

        let renderer = generate_renderer();

        for (input, expected_path) in test_cases.iter() {
            let output = util::remove_whitespace(renderer.render_sidebar_section(input)?);
            let expected = util::remove_whitespace(fs::read_to_string(expected_path)?);

            assert_eq!(output, expected);
        }

        Ok(())
    }
}
