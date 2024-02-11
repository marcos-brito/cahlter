#[derive(Clone, PartialEq, Debug)]
pub struct Section {
    pub title: String,
}

impl Section {
    pub fn new<S>(title: S) -> Self
    where
        S: Into<String>,
    {
        let title = title.into();

        Self { title }
    }
}
