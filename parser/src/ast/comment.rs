use crate::grammar::Rule;
use pest::iterators::Pair;

pub(crate) struct Comment {
    inner: String,
}

impl Comment {
    pub(crate) fn new() -> Self {
        Self {
            inner: String::new(),
        }
    }

    pub(crate) fn push(&mut self, pair: Pair<Rule>) {
        assert_eq!(pair.as_rule(), Rule::comment);

        let line = pair.as_str().strip_prefix("//").unwrap();
        let line = line.strip_prefix(' ').unwrap_or(line).trim_end();

        self.inner.push_str(line);
        self.inner.push('\n');
    }
}

impl From<Comment> for Option<String> {
    fn from(s: Comment) -> Self {
        if s.inner.is_empty() {
            None
        } else {
            Some(s.inner)
        }
    }
}
