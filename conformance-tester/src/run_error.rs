use anyhow::Error;

pub struct RunError {
    pub error: Error,
    pub stderr: Vec<u8>,
}

impl RunError {
    pub fn with_stderr(error: impl Into<Error>, stderr: Vec<u8>) -> Self {
        Self {
            error: error.into(),
            stderr,
        }
    }

    pub fn without_stderr(err: impl Into<Error>) -> Self {
        Self::with_stderr(err, Vec::new())
    }
}
