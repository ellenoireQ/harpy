#[macro_export]
macro_rules! compiler_error {
    ($span:expr, $msg:expr) => {
        $crate::logs::diagnostics::emit_diagnostic(
            $crate::logs::diagnostics::DiagnosticKind::Error,
            $span,
            $msg,
        );
    };
}

#[macro_export]
macro_rules! compiler_warning {
    ($span:expr, $msg:expr) => {
        $crate::logs::diagnostics::emit_diagnostic(
            $crate::logs::diagnostics::DiagnosticKind::Warning,
            $span,
            $msg,
        );
    };
}

#[macro_export]
macro_rules! compiler_info {
    ($span:expr, $msg:expr) => {
        $crate::logs::diagnostics::emit_diagnostic(
            $crate::logs::diagnostics::DiagnosticKind::Note,
            $span,
            $msg,
        );
    };
}
