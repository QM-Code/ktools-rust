use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    Logger, LoggerInner, OutputOptions, Severity, SourceLocation, TraceError, TraceResult,
};

fn format_source_label(source_path: &str) -> String {
    let file_name = source_path
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(source_path)
        .trim_end_matches(".rs");
    file_name.to_string()
}

fn format_function_label(function_name: &str) -> &str {
    function_name.rsplit("::").next().unwrap_or(function_name)
}

fn timestamp_label() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("[{}.{:06}]", now.as_secs(), now.subsec_micros())
}

pub(crate) fn emit_line(inner: &LoggerInner, prefix: &str, message: &str) -> TraceResult<()> {
    let _guard = inner
        .output_lock
        .lock()
        .map_err(|_| TraceError::new("ktrace output mutex is poisoned"))?;
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{prefix} {message}")
        .map_err(|error| TraceError::new(format!("failed to write trace output: {error}")))?;
    stdout
        .flush()
        .map_err(|error| TraceError::new(format!("failed to flush trace output: {error}")))?;
    Ok(())
}

fn build_source_part(output_options: OutputOptions, location: SourceLocation) -> Option<String> {
    if !output_options.filenames || location.file.is_empty() {
        return None;
    }

    let mut source = format!("[{}", format_source_label(location.file));
    if output_options.line_numbers && location.line > 0 {
        source.push(':');
        source.push_str(&location.line.to_string());
    }
    let function_label = format_function_label(location.function);
    if output_options.function_names && !function_label.is_empty() {
        source.push(':');
        source.push_str(function_label);
    }
    source.push(']');
    Some(source)
}

fn build_prefix(
    trace_namespace: &str,
    label: &str,
    location: SourceLocation,
    output_options: OutputOptions,
) -> String {
    let mut parts = Vec::new();
    if !trace_namespace.is_empty() {
        parts.push(format!("[{trace_namespace}]"));
    }
    if output_options.timestamps {
        parts.push(timestamp_label());
    }
    parts.push(format!("[{label}]"));
    if let Some(source) = build_source_part(output_options, location) {
        parts.push(source);
    }
    parts.join(" ")
}

impl Logger {
    pub(crate) fn build_trace_prefix(
        &self,
        trace_namespace: &str,
        channel: &str,
        location: SourceLocation,
    ) -> TraceResult<String> {
        Ok(build_prefix(
            trace_namespace,
            channel,
            location,
            self.get_output_options()?,
        ))
    }

    pub(crate) fn build_log_prefix(
        &self,
        trace_namespace: &str,
        severity: Severity,
        location: SourceLocation,
    ) -> TraceResult<String> {
        let label = match severity {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        };
        Ok(build_prefix(
            trace_namespace,
            label,
            location,
            self.get_output_options()?,
        ))
    }
}
