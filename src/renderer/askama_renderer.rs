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
}

impl Renderer for AskamaRenderer<'_> {
    fn render(&self) -> Result<String> {
        unimplemented!()
    }
}
