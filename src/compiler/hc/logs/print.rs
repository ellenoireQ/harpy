/// Emits a compiler error diagnostic for `span` with message `msg`.
///
/// This macro is `#[macro_export]`, so it can be called from other crates.
/// The `$crate` prefix ensures the internal path resolves to this crate even
/// when the macro is invoked externally.
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

/// Emits a compiler warning diagnostic for `span` with message `msg`.
///
/// Uses `$crate` so the path to `logs::diagnostics` remains robust when the
/// macro is expanded from outside this crate.
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

/// Emits a compiler note/info diagnostic for `span` with message `msg`.
///
/// This maps to `DiagnosticKind::Note` in the diagnostics backend.
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
