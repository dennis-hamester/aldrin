use crate::{Error, Warning};

#[derive(Debug, Default)]
pub(crate) struct Issues {
    errors: Vec<Error>,
    warnings: Vec<Warning>,
    other_warnings: Vec<Warning>,
}

impl Issues {
    pub(crate) fn add_error<E>(&mut self, e: E)
    where
        E: Into<Error>,
    {
        self.errors.push(e.into());
    }

    pub(crate) fn errors(&self) -> &[Error] {
        &self.errors
    }

    pub(crate) fn add_warning<W>(&mut self, w: W)
    where
        W: Into<Warning>,
    {
        self.warnings.push(w.into());
    }

    pub(crate) fn warnings(&self) -> &[Warning] {
        &self.warnings
    }

    pub(crate) fn add_other_warning<W>(&mut self, w: W)
    where
        W: Into<Warning>,
    {
        self.other_warnings.push(w.into());
    }

    pub(crate) fn other_warnings(&self) -> &[Warning] {
        &self.other_warnings
    }
}
