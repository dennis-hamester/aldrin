use super::Ident;
use crate::grammar::Rule;
use crate::Span;
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

        let span = Span::from_pair(&pair);

        let mut pairs = pair.into_inner();
        pairs.next().unwrap(); // Skip #.
        pairs.next().unwrap(); // Skip [.

        let pair = pairs.next().unwrap();
        let name = Ident::parse(pair);

        let mut att = Attribute {
            span,
            name,
            options: Vec::new(),
        };

        for pair in pairs {
            match pair.as_rule() {
                Rule::ident => att.options.push(Ident::parse(pair)),
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
