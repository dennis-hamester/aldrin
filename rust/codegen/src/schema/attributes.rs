use super::grammar::Rule;
use super::Ident;
use crate::error::Error;
use pest::iterators::{Pair, Pairs};

#[derive(Debug)]
pub(crate) struct Attributes {}

impl Attributes {
    pub fn new() -> Self {
        Attributes {}
    }

    pub fn from_pairs(pairs: &mut Pairs<Rule>) -> Result<Self, Error> {
        let mut attributes = Self::new();
        while let Some(pair) = pairs.peek() {
            if pair.as_rule() != Rule::attribute {
                break;
            }
            let pair = pairs.next().unwrap();
            attributes.extend(pair)?;
        }
        Ok(attributes)
    }

    fn extend(&mut self, pair: Pair<Rule>) -> Result<(), Error> {
        assert_eq!(pair.as_rule(), Rule::attribute);
        let mut pairs = pair.into_inner();
        let att = Ident::from_string(pairs.next().unwrap().as_str())?;
        match att.0.as_str() {
            // Ignore unknown attributes
            _ => {}
        }
        Ok(())
    }
}

struct AttOption {
    opt: Ident,
}

impl AttOption {
    fn from_att_option(pair: Pair<Rule>) -> Result<Self, Error> {
        assert_eq!(pair.as_rule(), Rule::att_option);
        let mut pairs = pair.into_inner();
        let opt = Ident::from_string(pairs.next().unwrap().as_str())?;
        Ok(AttOption { opt })
    }
}
