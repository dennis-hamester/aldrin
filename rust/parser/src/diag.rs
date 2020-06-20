pub trait Diagnostic {
    fn kind(&self) -> DiagnosticKind;
    fn schema_name(&self) -> &str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticKind {
    Error,
    Warning,
}
