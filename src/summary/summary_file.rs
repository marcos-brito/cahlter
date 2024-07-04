use super::{Summarizer, Summary};
use crate::util;
use crate::{Chapter, Item, Section};
use anyhow::Result;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[grammar = "summary/summary.pest"]
struct SummaryParser;

pub struct SummaryFileSummarizer {
    path: PathBuf,
}

impl SummaryFileSummarizer {
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    fn find_items(&self) -> Result<Vec<Item>> {
        let md = fs::read_to_string(&self.path)?;
        let summary = SummaryParser::parse(Rule::summary, &md)?;
        let mut chapter_number = "1".to_string();

        Ok(summary
            .filter_map(|line| match line.as_rule() {
                Rule::heading => {
                    let mut rules = line.into_inner();

                    Some(Item::from(Section::new(rules.next().unwrap().as_str())))
                }
                Rule::link => Some(Item::from(self.parse_link(line))),
                Rule::list => {
                    let item = Item::from(self.parse_list(line, chapter_number.clone()));
                    chapter_number = util::next_chapter_number(&chapter_number);

                    Some(item)
                }
                _ => None,
            })
            .collect())
    }

    fn parse_link(&self, rules: Pair<Rule>) -> Chapter {
        let mut rules = rules.into_inner();
        let title = rules.next().unwrap().as_str();
        let content = rules.next().unwrap().as_str();

        Chapter::new(
            title,
            "",
            self.path.parent().unwrap_or(Path::new("")).join(content),
            vec![],
        )
    }

    fn parse_list(&self, rules: Pair<Rule>, chapter_number: String) -> Chapter {
        let mut rules = rules.into_inner();
        let mut chapter = self.parse_link(rules.next().unwrap());

        let mut number = chapter_number.clone() + ".0";
        chapter.subchapters = rules
            .map(|rule| {
                number = util::next_chapter_number(&number);

                self.parse_list(rule, number.clone())
            })
            .collect();
        chapter.number = chapter_number.clone();

        chapter
    }
}

impl Summarizer for SummaryFileSummarizer {
    fn summarize(&self) -> Result<Summary> {
        Ok(Summary::new(self.find_items()?))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use tempfile::tempdir;

    #[test]
    fn test_find_chapters() -> Result<()> {
        let dir = tempdir()?;
        let expected = vec![
            Item::from(Chapter::new(
                "Intro",
                "",
                dir.path().join("summary.md").join("./intro.md"),
                vec![],
            )),
            Item::from(Chapter::new(
                "Preface",
                "",
                dir.path().join("summary.md").join("./preface.md"),
                vec![],
            )),
            Item::from(Chapter::new(
                "Chapter 1",
                "1",
                dir.path().join("summary.md").join("./chapter1.md"),
                vec![],
            )),
            Item::from(Section::new("Section")),
            Item::from(Chapter::new(
                "Chapter 2",
                "2",
                dir.path().join("summary.md").join("./chapter2.md"),
                vec![],
            )),
        ];

        fs::write(
            dir.path().join("summary.md"),
            r#"
[Intro](./intro.md)
[Preface](./preface.md)

- [Chapter 1](./chapter1.md)

# Section

- [Chapter 2](./chapter2.md)
"#,
        )?;

        let summarizer = SummaryFileSummarizer::new(dir.path().join("summary.md"));

        assert_eq!(expected, summarizer.find_items()?);

        Ok(())
    }

    #[test]
    fn test_find_chapters_nested() -> Result<()> {
        let dir = tempdir()?;
        let expected = vec![
            Item::from(Chapter::new(
                "Chapter 1",
                "1",
                dir.path().join("summary.md").join("./chapter1.md"),
                vec![
                    Chapter::new(
                        "Chapter 1.1",
                        "1.1",
                        dir.path()
                            .join("summary.md")
                            .join("./chapter1/chapter1.1.md"),
                        vec![Chapter::new(
                            "Chapter 1.1.1",
                            "1.1.1",
                            dir.path()
                                .join("summary.md")
                                .join("./chapter1/chapter1.1/chapter1.1.1.md"),
                            vec![],
                        )],
                    ),
                    Chapter::new(
                        "Chapter 1.2",
                        "1.2",
                        dir.path()
                            .join("summary.md")
                            .join("./chapter1/chapter1.2.md"),
                        vec![],
                    ),
                ],
            )),
            Item::from(Chapter::new(
                "Chapter 2",
                "2",
                dir.path().join("summary.md").join("chapter2.md"),
                vec![],
            )),
            Item::from(Section::new("Section")),
        ];

        fs::write(
            dir.path().join("summary.md"),
            r#"
- [Chapter 1](./chapter1.md)
    - [Chapter 1.1](./chapter1/chapter1.1.md)
        - [Chapter 1.1.1](./chapter1/chapter1.1/chapter1.1.1.md)
    - [Chapter 1.2](./chapter1/chapter1.2.md)

- [Chapter 2](./chapter2.md)

# Section
"#,
        )?;

        let summarizer = SummaryFileSummarizer::new(dir.path().join("summary.md"));

        assert_eq!(expected, summarizer.find_items()?);

        Ok(())
    }
}
