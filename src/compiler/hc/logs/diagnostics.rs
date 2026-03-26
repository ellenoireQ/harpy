#[derive(Debug, Clone)]
pub struct Span {
    pub file: String,
    pub line: usize,
    pub column: usize,
}
pub enum DiagnosticKind {
    Error,
    Warning,
    Note,
}

pub fn emit_diagnostic(kind: DiagnosticKind, span: &Span, message: &str) {
    let kind_str = match kind {
        DiagnosticKind::Error => "error",
        DiagnosticKind::Warning => "warning",
        DiagnosticKind::Note => "note",
    };

    eprintln!(
        "{}:{}:{}: {}: {}",
        span.file, span.line, span.column, kind_str, message
    );
}
