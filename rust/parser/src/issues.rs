use crate::{Error, Warning};

#[derive(Debug, Default)]
pub(crate) struct Issues {
    errors: Vec<Error>,
    warnings: Vec<Warning>,
}

impl Issues {
    pub fn add_error<E>(&mut self, e: E) -> &mut Self
    where
        E: Into<Error>,
    {
        self.errors.push(e.into());
        self
    }

    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    pub fn add_warning<E>(&mut self, e: E) -> &mut Self
    where
        E: Into<Warning>,
    {
        self.warnings.push(e.into());
        self
    }

    pub fn warnings(&self) -> &[Warning] {
        &self.warnings
    }
}
