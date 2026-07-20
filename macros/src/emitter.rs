use proc_macro2::Span;
use std::fmt::Display;
use syn::Error;

pub(crate) struct Emitter {
    err: Option<Error>,
}

impl Emitter {
    pub(crate) fn new() -> Self {
        Self { err: None }
    }

    pub(crate) fn add(&mut self, msg: impl Display) {
        let err = Error::new(Span::call_site(), msg);

        match self.err {
            Some(ref mut e) => e.combine(err),
            None => self.err = Some(err),
        }
    }

    pub(crate) fn into_result(self) -> Result<(), Error> {
        match self.err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}
