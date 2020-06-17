use super::Error;

#[derive(Debug)]
pub struct IoError {
    schema_name: String,
    err: std::io::Error,
}

impl IoError {
    pub(crate) fn new<S>(schema_name: S, err: std::io::Error) -> Self
    where
        S: Into<String>,
    {
        IoError {
            schema_name: schema_name.into(),
            err,
        }
    }

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn io_error(&self) -> &std::io::Error {
        &self.err
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::Io(e)
    }
}
