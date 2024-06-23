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
    theme: &'a String,
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
        let theme = &self.context.config.appearance.default_theme;

        let markdown = fs::read_to_string(&chapter.content).unwrap();
        let parser = pulldown_cmark::Parser::new(&markdown);
        let mut html = String::new();

        pulldown_cmark::html::push_html(&mut html, parser);

        let index = Page {
            theme: &theme,
            header: &header,
            sidebar: &sidebar,
            content: &html,
        };

        return Ok(index.render()?);
    }
}
