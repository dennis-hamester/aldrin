use super::grammar::Rule;
use super::Ident;
use crate::error::Error;
use pest::iterators::{Pair, Pairs};

#[derive(Debug)]
pub(crate) struct Attributes {
    pub rust: RustAttributes,
}

impl Attributes {
    pub fn new() -> Self {
        Attributes {
            rust: RustAttributes::new(),
        }
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
        #[allow(clippy::single_match)] // Remove this when adding a second attribute
        match att.0.as_str() {
            "rust" => self.rust.extend(pairs)?,
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

#[derive(Debug)]
pub(crate) struct RustAttributes {
    pub impl_copy: bool,
    pub impl_partial_eq: bool,
    pub impl_eq: bool,
    pub impl_partial_ord: bool,
    pub impl_ord: bool,
}

impl RustAttributes {
    fn new() -> Self {
        RustAttributes {
            impl_copy: false,
            impl_partial_eq: false,
            impl_eq: false,
            impl_partial_ord: false,
            impl_ord: false,
        }
    }

    fn extend(&mut self, pairs: Pairs<Rule>) -> Result<(), Error> {
        for pair in pairs {
            let opt = AttOption::from_att_option(pair)?;
            match opt.opt.0.as_str() {
                "impl_copy" => self.impl_copy = true,
                "impl_partial_eq" => self.impl_partial_eq = true,
                "impl_eq" => self.impl_eq = true,
                "impl_partial_ord" => self.impl_partial_ord = true,
                "impl_ord" => self.impl_ord = true,
                // Ignore unknown options
                _ => {}
            }
        }
        Ok(())
    }
}
