use super::{Renderer, RendererContext};
use crate::config::Link;
use crate::{Chapter, Item, Section};
use anyhow::{anyhow, Context, Result};
use askama::Template;
use std::fs;
use std::path::{Path, PathBuf};

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
    theme: &'a String,
    header: &'a String,
    sidebar: &'a String,
    content: &'a String,
    custom_css: &'a Vec<String>,
    themes: &'a Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AskamaRenderer {
    context: RendererContext,
}

impl AskamaRenderer {
    pub fn new(context: RendererContext) -> Self {
        Self { context }
    }

    pub fn render_header(&self) -> Result<String> {
        let links = self.context.config.links.clone();
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

        let target = self.get_chapter_target(chapter.content.clone())?;
        let title = match self.context.config.general.enumerate {
            true => format!("{} {}", chapter.number, chapter.title),
            false => chapter.title.clone(),
        };

        let sidebar_chapter = SidebarChapter {
            title: &title,
            subchapters: &subchapters,
            target: &target,
        };

        Ok(sidebar_chapter.render()?)
    }

    fn get_chapter_target(&self, path: PathBuf) -> Result<String> {
        Ok("/".to_string()
            + path
                .strip_prefix(&self.context.src_dir)
                .and_then(|url| Ok(url.with_extension("html")))
                .with_context(|| anyhow!("Failed to create the url for {}", path.display()))?
                .to_string_lossy()
                .as_ref())
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
        let mut custom_css = Vec::new();

        for css in self.context.config.appearance.custom.iter() {
            let file_name = Path::new(css)
                .file_name()
                .with_context(|| anyhow!("Failed to extract the file name from {css}"))?;

            custom_css.push("/".to_string() + &file_name.to_string_lossy().to_string());
        }

        let markdown = fs::read_to_string(&chapter.content)
            .with_context(|| anyhow!("Failed to read contents of {}", chapter.content.display()))?;
        let parser = pulldown_cmark::Parser::new(&markdown);
        let mut html = String::new();

        pulldown_cmark::html::push_html(&mut html, parser);

        let index = Page {
            theme: &self.context.config.appearance.default_theme,
            header: &header,
            sidebar: &sidebar,
            content: &html,
            custom_css: &custom_css,
            themes: &self.context.config.appearance.themes,
        };

        return Ok(index.render()?);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_get_chapter_target() -> Result<()> {
        let tempdir = tempdir()?;
        let context = RendererContext::new(
            crate::Content::new(tempdir.path())?,
            crate::config::Config::default(),
            PathBuf::from("/some/dir/src"),
        );
        let renderer = AskamaRenderer::new(context);
        let tests = vec![
            ("/some/dir/src/file.txt", "/file.html"),
            ("/some/dir/src/dir/file", "/dir/file.html"),
        ];

        for test in tests.iter() {
            assert_eq!(
                renderer.get_chapter_target(PathBuf::from(test.0))?,
                test.1.to_string()
            );
        }

        Ok(())
    }
}
