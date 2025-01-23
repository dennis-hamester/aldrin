use super::kw;
use syn::parse::{Parse, ParseStream};
use syn::{Result, Token, Type};

pub(super) struct FnBody {
    args: Option<Type>,
    ok: Option<Type>,
    err: Option<Type>,
}

impl FnBody {
    pub fn empty() -> Self {
        Self {
            args: None,
            ok: None,
            err: None,
        }
    }

    pub fn parse_simplified(input: ParseStream) -> Result<Self> {
        input.parse::<Token![=]>()?;
        let ok = input.parse()?;

        Ok(Self {
            args: None,
            ok: Some(ok),
            err: None,
        })
    }

    pub fn args(&self) -> Option<&Type> {
        self.args.as_ref()
    }

    pub fn ok(&self) -> Option<&Type> {
        self.ok.as_ref()
    }

    pub fn err(&self) -> Option<&Type> {
        self.err.as_ref()
    }
}

impl Parse for FnBody {
    fn parse(input: ParseStream) -> Result<Self> {
        let args = if input.parse::<kw::args>().is_ok() {
            input.parse::<Token![=]>()?;
            let args = input.parse()?;
            input.parse::<Token![;]>()?;
            Some(args)
        } else {
            None
        };

        let ok = if input.parse::<kw::ok>().is_ok() {
            input.parse::<Token![=]>()?;
            let ok = input.parse()?;
            input.parse::<Token![;]>()?;
            Some(ok)
        } else {
            None
        };

        let err = if input.parse::<kw::err>().is_ok() {
            input.parse::<Token![=]>()?;
            let err = input.parse()?;
            input.parse::<Token![;]>()?;
            Some(err)
        } else {
            None
        };

        Ok(Self { args, ok, err })
    }
}
