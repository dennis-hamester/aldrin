use crate::{Error, Warning};

#[derive(Debug, Default)]
pub(crate) struct Issues {
    errors: Vec<Error>,
    warnings: Vec<Warning>,
}

impl Issues {
    pub fn add_error<E>(&mut self, e: E)
    where
        E: Into<Error>,
    {
        self.errors.push(e.into());
    }

    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    pub fn add_warning<W>(&mut self, w: W)
    where
        W: Into<Warning>,
    {
        self.warnings.push(w.into());
    }

    pub fn warnings(&self) -> &[Warning] {
        &self.warnings
    }
}
