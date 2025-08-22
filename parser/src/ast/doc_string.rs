use crate::grammar::Rule;
use pest::iterators::Pair;

pub(crate) struct DocString {
    inner: String,
}

impl DocString {
    pub(crate) fn new() -> Self {
        Self {
            inner: String::new(),
        }
    }

    pub(crate) fn push(&mut self, pair: Pair<Rule>) {
        assert_eq!(pair.as_rule(), Rule::doc_string);
        self.push_impl(pair, "///");
    }

    pub(crate) fn push_inline(&mut self, pair: Pair<Rule>) {
        assert_eq!(pair.as_rule(), Rule::doc_string_inline);
        self.push_impl(pair, "//!");
    }

    fn push_impl(&mut self, pair: Pair<Rule>, prefix: &'static str) {
        let line = pair.as_str().strip_prefix(prefix).unwrap();
        let line = line.strip_prefix(' ').unwrap_or(line).trim_end();

        self.inner.push_str(line);
        self.inner.push('\n');
    }
}

impl From<DocString> for Option<String> {
    fn from(s: DocString) -> Self {
        if s.inner.is_empty() {
            None
        } else {
            Some(s.inner)
        }
    }
}
