pub use coloc_macro::coloc;

#[derive(Clone, Debug, PartialEq)]
pub enum Inline {
    Text(String),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Link { href: String, children: Vec<Inline> },
    Code(String),
    LineBreak,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    Heading { level: u8, children: Vec<Inline> },
    Paragraph(Vec<Inline>),
}

impl From<&str> for Inline {
    fn from(s: &str) -> Self {
        Inline::Text(s.to_string())
    }
}

impl Inline {
    pub fn to_markdown(&self) -> String {
        match self {
            Inline::Text(t) => t.clone(),
            Inline::Bold(children) => {
                let inner: String = children.iter().map(Inline::to_markdown).collect();
                format!("**{inner}**")
            }
            Inline::Italic(children) => {
                let inner: String = children.iter().map(Inline::to_markdown).collect();
                format!("*{inner}*")
            }
            Inline::Link { href, children } => {
                let inner: String = children.iter().map(Inline::to_markdown).collect();
                format!("[{inner}]({href})")
            }
            Inline::Code(c) => format!("`{c}`"),
            Inline::LineBreak => "\n".to_string(),
        }
    }
}

impl Block {
    pub fn to_markdown(&self) -> String {
        match self {
            Block::Heading { level, children } => {
                let prefix = "#".repeat(*level as usize);
                let inner: String = children.iter().map(Inline::to_markdown).collect();
                format!("{prefix} {inner}\n\n")
            }
            Block::Paragraph(children) => {
                let inner: String = children.iter().map(Inline::to_markdown).collect();
                format!("{inner}\n\n")
            }
        }
    }
}

#[macro_export]
macro_rules! p {
    ($($e:expr),* $(,)?) => {
        $crate::Block::Paragraph(vec![$($e.into()),*])
    };
}

#[macro_export]
macro_rules! link {
    ($text:expr, $href:expr $(,)?) => {
        $crate::Inline::Link {
            href: $href.to_string(),
            children: vec![$crate::Inline::from($text)],
        }
    };
}
