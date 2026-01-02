use super::Ident;
use crate::Span;
use crate::grammar::Rule;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct Attribute {
    span: Span,
    name: Ident,
    options: Vec<Ident>,
}

impl Attribute {
    pub(crate) fn parse(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::attribute);
        Self::parse_impl(pair)
    }

    pub(crate) fn parse_inline(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::attribute_inline);
        Self::parse_impl(pair)
    }

    fn parse_impl(pair: Pair<Rule>) -> Self {
        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();
        pairs.next().unwrap(); // Skip [.

        let pair = pairs.next().unwrap();
        let name = Ident::parse(&pair);

        let mut att = Self {
            span,
            name,
            options: Vec::new(),
        };

        for pair in pairs {
            #[expect(clippy::wildcard_enum_match_arm)]
            match pair.as_rule() {
                Rule::ident => att.options.push(Ident::parse(&pair)),

                Rule::tok_par_open
                | Rule::tok_par_close
                | Rule::tok_comma
                | Rule::tok_squ_close => {}

                _ => unreachable!(),
            }
        }

        att
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn options(&self) -> &[Ident] {
        &self.options
    }
}
