mod colors;
pub mod demo;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::io::{self, Write};
use std::sync::{Arc, Mutex, Weak};
use std::time::{SystemTime, UNIX_EPOCH};

pub use colors::{available_color_names, color};

pub type ColorId = u16;
pub const DEFAULT_COLOR: ColorId = 0xFFFF;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OutputOptions {
    pub filenames: bool,
    pub line_numbers: bool,
    pub function_names: bool,
    pub timestamps: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: &'static str,
    pub line: u32,
    pub function: &'static str,
}

impl SourceLocation {
    pub const fn new(file: &'static str, line: u32, function: &'static str) -> Self {
        Self {
            file,
            line,
            function,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceError {
    message: String,
}

impl TraceError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for TraceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for TraceError {}

type TraceResult<T> = Result<T, TraceError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ChannelSpec {
    name: String,
    color: ColorId,
}

struct TraceLoggerData {
    trace_namespace: String,
    channels: Mutex<Vec<ChannelSpec>>,
    attached_logger: Mutex<Option<Weak<LoggerInner>>>,
    changed_keys: Mutex<HashMap<String, String>>,
}

#[derive(Default)]
struct RegistryState {
    namespaces: BTreeSet<String>,
    channels_by_namespace: HashMap<String, Vec<String>>,
    channel_colors_by_namespace: HashMap<String, HashMap<String, ColorId>>,
    attached_trace_loggers: Vec<Arc<TraceLoggerData>>,
}

struct LoggerInner {
    enabled_channel_keys: Mutex<HashSet<String>>,
    output_options: Mutex<OutputOptions>,
    registry: Mutex<RegistryState>,
    output_lock: Mutex<()>,
}

#[derive(Clone)]
pub struct TraceLogger {
    data: Arc<TraceLoggerData>,
}

#[derive(Clone)]
pub struct Logger {
    inner: Arc<LoggerInner>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Selector {
    any_namespace: bool,
    trace_namespace: String,
    channel_tokens: Vec<String>,
    include_top_level: bool,
}

#[derive(Default)]
struct SelectorResolution {
    channel_keys: Vec<String>,
    unmatched_selectors: Vec<String>,
}

fn trim_whitespace(value: &str) -> String {
    value.trim().to_string()
}

fn is_selector_identifier(token: &str) -> bool {
    !token.is_empty()
        && token
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

fn split_channel_path(channel: &str) -> Option<Vec<&str>> {
    if channel.is_empty() {
        return None;
    }
    let tokens = channel.split('.').collect::<Vec<_>>();
    if tokens.is_empty() || tokens.len() > 3 {
        return None;
    }
    if tokens
        .iter()
        .any(|token| token.is_empty() || !is_selector_identifier(token))
    {
        return None;
    }
    Some(tokens)
}

fn is_valid_channel_path(channel: &str) -> bool {
    split_channel_path(channel).is_some()
}

fn make_qualified_channel_key(trace_namespace: &str, channel: &str) -> String {
    if trace_namespace.is_empty() || channel.is_empty() {
        return String::new();
    }
    format!("{trace_namespace}.{channel}")
}

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

fn emit_line(inner: &LoggerInner, prefix: &str, message: &str) -> TraceResult<()> {
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

fn split_by_top_level_commas(value: &str) -> Result<Vec<String>, String> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut brace_depth = 0i32;

    for (index, ch) in value.char_indices() {
        match ch {
            '{' => brace_depth += 1,
            '}' => {
                if brace_depth == 0 {
                    return Err("unmatched '}'".to_string());
                }
                brace_depth -= 1;
            }
            ',' if brace_depth == 0 => {
                parts.push(trim_whitespace(&value[start..index]));
                start = index + 1;
            }
            _ => {}
        }
    }

    if brace_depth != 0 {
        return Err("unmatched '{'".to_string());
    }

    parts.push(trim_whitespace(&value[start..]));
    Ok(parts)
}

fn expand_brace_expression(value: &str) -> Result<Vec<String>, String> {
    let Some(open) = value.find('{') else {
        return Ok(vec![value.to_string()]);
    };

    let mut depth = 0i32;
    let mut close = None;
    for (index, ch) in value.char_indices().skip(open) {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(index);
                    break;
                }
            }
            _ => {}
        }
    }

    let Some(close) = close else {
        return Err("unmatched '{'".to_string());
    };

    let prefix = &value[..open];
    let suffix = &value[close + 1..];
    let inside = &value[open + 1..close];
    if inside.is_empty() {
        return Err("empty brace group".to_string());
    }

    let alternatives = split_by_top_level_commas(inside)?;
    let mut expanded = Vec::new();
    for alternative in alternatives {
        if alternative.is_empty() {
            return Err("empty brace alternative".to_string());
        }
        let combined = format!("{prefix}{alternative}{suffix}");
        expanded.extend(expand_brace_expression(&combined)?);
    }
    Ok(expanded)
}

fn parse_selector_channel_pattern(expression: &str) -> Result<(Vec<String>, bool), String> {
    if expression.is_empty() {
        return Err("missing channel expression".to_string());
    }

    let tokens = expression
        .split('.')
        .map(str::trim)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if tokens.is_empty() || tokens.len() > 3 {
        return Err("channel depth exceeds 3".to_string());
    }
    if tokens
        .iter()
        .any(|token| token.is_empty() || (token != "*" && !is_selector_identifier(token)))
    {
        return Err("invalid channel token".to_string());
    }

    let include_top_level = tokens.len() == 2 && tokens[0] == "*" && tokens[1] == "*";
    Ok((tokens, include_top_level))
}

fn parse_selector_expression(raw_token: &str, local_namespace: &str) -> Result<Selector, String> {
    let Some(dot) = raw_token.find('.') else {
        return Err("did you mean '.*'?".to_string());
    };

    let namespace_token = &raw_token[..dot];
    let channel_pattern = &raw_token[dot + 1..];
    let (channel_tokens, include_top_level) = parse_selector_channel_pattern(channel_pattern)?;

    let (any_namespace, trace_namespace) = if namespace_token == "*" {
        (true, String::new())
    } else if namespace_token.is_empty() {
        let namespace_name = trim_whitespace(local_namespace);
        if !is_selector_identifier(&namespace_name) {
            return Err("missing namespace".to_string());
        }
        (false, namespace_name)
    } else if is_selector_identifier(namespace_token) {
        (false, namespace_token.to_string())
    } else {
        return Err(format!("invalid namespace '{namespace_token}'"));
    };

    Ok(Selector {
        any_namespace,
        trace_namespace,
        channel_tokens,
        include_top_level,
    })
}

fn format_selector(selector: &Selector) -> String {
    let namespace_label = if selector.any_namespace {
        "*".to_string()
    } else {
        selector.trace_namespace.clone()
    };
    format!("{namespace_label}.{}", selector.channel_tokens.join("."))
}

fn parse_selector_list(list: &str, local_namespace: &str) -> TraceResult<Vec<Selector>> {
    let selector_tokens = split_by_top_level_commas(list)
        .map_err(|reason| TraceError::new(format!("invalid trace selector '{list}' ({reason})")))?;

    let mut selectors = Vec::new();
    let mut invalid_tokens = Vec::new();
    for token in selector_tokens {
        let name = trim_whitespace(&token);
        if name.is_empty() {
            invalid_tokens.push("<empty>".to_string());
            continue;
        }

        match expand_brace_expression(&name) {
            Ok(expanded) => {
                for item in expanded {
                    match parse_selector_expression(&item, local_namespace) {
                        Ok(selector) => selectors.push(selector),
                        Err(reason) => invalid_tokens.push(format!("{item} ({reason})")),
                    }
                }
            }
            Err(reason) => invalid_tokens.push(format!("{name} ({reason})")),
        }
    }

    if invalid_tokens.is_empty() {
        return Ok(selectors);
    }

    let details = invalid_tokens
        .iter()
        .map(|token| format!("'{token}'"))
        .collect::<Vec<_>>()
        .join(", ");
    Err(TraceError::new(format!(
        "Invalid trace selector(s): {details}"
    )))
}

fn matches_selector(selector: &Selector, trace_namespace: &str, channel: &str) -> bool {
    if !selector.any_namespace && selector.trace_namespace != trace_namespace {
        return false;
    }

    let Some(channel_parts) = split_channel_path(channel) else {
        return false;
    };

    match selector.channel_tokens.len() {
        1 => {
            channel_parts.len() == 1
                && (selector.channel_tokens[0] == "*"
                    || selector.channel_tokens[0] == channel_parts[0])
        }
        2 => {
            if channel_parts.len() == 1 && selector.include_top_level {
                return true;
            }
            channel_parts.len() == 2
                && (selector.channel_tokens[0] == "*"
                    || selector.channel_tokens[0] == channel_parts[0])
                && (selector.channel_tokens[1] == "*"
                    || selector.channel_tokens[1] == channel_parts[1])
        }
        3 => {
            let include_up_to_depth_three =
                selector.channel_tokens.iter().all(|token| token == "*");
            if include_up_to_depth_three {
                return (1..=3).contains(&channel_parts.len());
            }
            channel_parts.len() == 3
                && (selector.channel_tokens[0] == "*"
                    || selector.channel_tokens[0] == channel_parts[0])
                && (selector.channel_tokens[1] == "*"
                    || selector.channel_tokens[1] == channel_parts[1])
                && (selector.channel_tokens[2] == "*"
                    || selector.channel_tokens[2] == channel_parts[2])
        }
        _ => false,
    }
}

fn parse_exact_selector(
    selector_text: &str,
    local_namespace: &str,
) -> TraceResult<(String, String)> {
    let selector = trim_whitespace(selector_text);
    let Some(dot) = selector.find('.') else {
        return Err(TraceError::new(format!(
            "invalid channel selector '{selector}' (expected namespace.channel or .channel)"
        )));
    };

    let trace_namespace = if dot == 0 {
        trim_whitespace(local_namespace)
    } else {
        selector[..dot].to_string()
    };
    let channel = selector[dot + 1..].to_string();

    if !is_selector_identifier(&trace_namespace) {
        return Err(TraceError::new(format!(
            "invalid trace namespace '{trace_namespace}'"
        )));
    }
    if !is_valid_channel_path(&channel) {
        return Err(TraceError::new(format!(
            "invalid trace channel '{channel}'"
        )));
    }

    Ok((trace_namespace, channel))
}

fn merge_color(
    existing_color: ColorId,
    new_color: ColorId,
    qualified_name: &str,
) -> TraceResult<ColorId> {
    if new_color == DEFAULT_COLOR {
        return Ok(existing_color);
    }
    if new_color > 255 {
        return Err(TraceError::new(format!(
            "invalid trace color id '{new_color}'"
        )));
    }
    if existing_color == DEFAULT_COLOR || existing_color == new_color {
        return Ok(new_color);
    }
    Err(TraceError::new(format!(
        "conflicting trace color for '{qualified_name}'"
    )))
}

impl TraceLogger {
    pub fn new(trace_namespace: impl AsRef<str>) -> TraceResult<Self> {
        let trace_namespace = trim_whitespace(trace_namespace.as_ref());
        if !is_selector_identifier(&trace_namespace) {
            return Err(TraceError::new(format!(
                "invalid trace namespace '{trace_namespace}'"
            )));
        }

        Ok(Self {
            data: Arc::new(TraceLoggerData {
                trace_namespace,
                channels: Mutex::new(Vec::new()),
                attached_logger: Mutex::new(None),
                changed_keys: Mutex::new(HashMap::new()),
            }),
        })
    }

    pub fn add_channel(&self, channel: impl AsRef<str>, color: ColorId) -> TraceResult<()> {
        let channel_name = trim_whitespace(channel.as_ref());
        if !is_valid_channel_path(&channel_name) {
            return Err(TraceError::new(format!(
                "invalid trace channel '{channel_name}'"
            )));
        }
        if color != DEFAULT_COLOR && color > 255 {
            return Err(TraceError::new(format!("invalid trace color id '{color}'")));
        }

        let mut channels = self
            .data
            .channels
            .lock()
            .map_err(|_| TraceError::new("ktrace channel mutex is poisoned"))?;

        if let Some(parent_index) = channel_name.rfind('.') {
            let parent = &channel_name[..parent_index];
            if !channels.iter().any(|existing| existing.name == parent) {
                return Err(TraceError::new(format!(
                    "cannot add unparented trace channel '{channel_name}' (missing parent '{parent}')"
                )));
            }
        }

        if let Some(existing) = channels
            .iter_mut()
            .find(|existing| existing.name == channel_name)
        {
            existing.color = merge_color(
                existing.color,
                color,
                &format!("{}.{}", self.data.trace_namespace, channel_name),
            )?;
            return Ok(());
        }

        channels.push(ChannelSpec {
            name: channel_name,
            color,
        });
        Ok(())
    }

    pub fn namespace(&self) -> &str {
        &self.data.trace_namespace
    }

    fn attached_logger(&self) -> Option<Arc<LoggerInner>> {
        self.data
            .attached_logger
            .lock()
            .ok()
            .and_then(|attached| attached.clone())
            .and_then(|attached| attached.upgrade())
    }

    pub fn should_trace_channel(&self, channel: impl AsRef<str>) -> bool {
        let channel_name = trim_whitespace(channel.as_ref());
        if !is_valid_channel_path(&channel_name) {
            return false;
        }
        self.attached_logger()
            .map(|logger| {
                Logger { inner: logger }.is_trace_channel_enabled(self.namespace(), &channel_name)
            })
            .unwrap_or(false)
    }

    #[track_caller]
    pub fn trace(&self, channel: impl AsRef<str>, message: impl Into<String>) -> TraceResult<()> {
        let location = std::panic::Location::caller();
        self.trace_with_location(
            channel.as_ref(),
            SourceLocation::new(location.file(), location.line(), ""),
            message.into(),
        )
    }

    pub fn trace_with_location(
        &self,
        channel: &str,
        location: SourceLocation,
        message: String,
    ) -> TraceResult<()> {
        let channel_name = trim_whitespace(channel);
        if !is_valid_channel_path(&channel_name) {
            return Err(TraceError::new(format!(
                "invalid trace channel '{channel_name}'"
            )));
        }

        let Some(logger) = self.attached_logger() else {
            return Ok(());
        };
        let logger_handle = Logger { inner: logger };
        if !logger_handle.is_trace_channel_enabled(self.namespace(), &channel_name) {
            return Ok(());
        }

        let prefix = logger_handle.build_trace_prefix(self.namespace(), &channel_name, location)?;
        emit_line(&logger_handle.inner, &prefix, &message)
    }

    #[track_caller]
    pub fn trace_changed(
        &self,
        channel: impl AsRef<str>,
        key: impl Into<String>,
        message: impl Into<String>,
    ) -> TraceResult<()> {
        let location = std::panic::Location::caller();
        self.trace_changed_with_location(
            channel.as_ref(),
            key.into(),
            SourceLocation::new(location.file(), location.line(), ""),
            message.into(),
        )
    }

    pub fn trace_changed_with_location(
        &self,
        channel: &str,
        key: String,
        location: SourceLocation,
        message: String,
    ) -> TraceResult<()> {
        let channel_name = trim_whitespace(channel);
        if !is_valid_channel_path(&channel_name) {
            return Err(TraceError::new(format!(
                "invalid trace channel '{channel_name}'"
            )));
        }

        let site_key = format!(
            "{}:{}:{}:{}",
            location.file, location.line, location.function, channel_name
        );
        let mut changed_keys = self
            .data
            .changed_keys
            .lock()
            .map_err(|_| TraceError::new("ktrace changed-keys mutex is poisoned"))?;
        if changed_keys.get(&site_key).map(String::as_str) == Some(key.as_str()) {
            return Ok(());
        }
        changed_keys.insert(site_key, key);
        drop(changed_keys);

        self.trace_with_location(&channel_name, location, message)
    }

    #[track_caller]
    pub fn info(&self, message: impl Into<String>) -> TraceResult<()> {
        let location = std::panic::Location::caller();
        self.log_with_location(
            Severity::Info,
            SourceLocation::new(location.file(), location.line(), ""),
            message.into(),
        )
    }

    #[track_caller]
    pub fn warn(&self, message: impl Into<String>) -> TraceResult<()> {
        let location = std::panic::Location::caller();
        self.log_with_location(
            Severity::Warning,
            SourceLocation::new(location.file(), location.line(), ""),
            message.into(),
        )
    }

    #[track_caller]
    pub fn error(&self, message: impl Into<String>) -> TraceResult<()> {
        let location = std::panic::Location::caller();
        self.log_with_location(
            Severity::Error,
            SourceLocation::new(location.file(), location.line(), ""),
            message.into(),
        )
    }

    pub fn log_with_location(
        &self,
        severity: Severity,
        location: SourceLocation,
        message: String,
    ) -> TraceResult<()> {
        let Some(logger) = self.attached_logger() else {
            return Ok(());
        };
        let logger_handle = Logger { inner: logger };
        let prefix = logger_handle.build_log_prefix(self.namespace(), severity, location)?;
        emit_line(&logger_handle.inner, &prefix, &message)
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Logger {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(LoggerInner {
                enabled_channel_keys: Mutex::new(HashSet::new()),
                output_options: Mutex::new(OutputOptions::default()),
                registry: Mutex::new(RegistryState::default()),
                output_lock: Mutex::new(()),
            }),
        }
    }

    pub fn add_trace_logger(&self, logger: TraceLogger) -> TraceResult<()> {
        let mut attached_logger = logger
            .data
            .attached_logger
            .lock()
            .map_err(|_| TraceError::new("ktrace attachment mutex is poisoned"))?;
        if let Some(existing) = attached_logger.as_ref().and_then(Weak::upgrade) {
            if !Arc::ptr_eq(&existing, &self.inner) {
                return Err(TraceError::new(
                    "trace logger is already attached to another logger",
                ));
            }
        }

        let channels = logger
            .data
            .channels
            .lock()
            .map_err(|_| TraceError::new("ktrace channel mutex is poisoned"))?
            .clone();

        let mut registry = self
            .inner
            .registry
            .lock()
            .map_err(|_| TraceError::new("ktrace registry mutex is poisoned"))?;
        registry
            .namespaces
            .insert(logger.data.trace_namespace.clone());
        let mut registered_channels = registry
            .channels_by_namespace
            .remove(&logger.data.trace_namespace)
            .unwrap_or_default();
        let mut registered_colors = registry
            .channel_colors_by_namespace
            .remove(&logger.data.trace_namespace)
            .unwrap_or_default();

        for channel in channels {
            if let Some(parent_index) = channel.name.rfind('.') {
                let parent = &channel.name[..parent_index];
                if !registered_channels
                    .iter()
                    .any(|existing| existing == parent)
                {
                    return Err(TraceError::new(format!(
                        "cannot register unparented trace channel '{}' (missing parent '{}')",
                        channel.name, parent
                    )));
                }
            }

            if !registered_channels
                .iter()
                .any(|existing| existing == &channel.name)
            {
                registered_channels.push(channel.name.clone());
            }

            let existing_color = *registered_colors
                .get(&channel.name)
                .unwrap_or(&DEFAULT_COLOR);
            let merged_color = merge_color(
                existing_color,
                channel.color,
                &format!("{}.{}", logger.data.trace_namespace, channel.name),
            )?;
            if merged_color != DEFAULT_COLOR {
                registered_colors.insert(channel.name.clone(), merged_color);
            }
        }

        registry
            .channels_by_namespace
            .insert(logger.data.trace_namespace.clone(), registered_channels);
        registry
            .channel_colors_by_namespace
            .insert(logger.data.trace_namespace.clone(), registered_colors);

        if !registry
            .attached_trace_loggers
            .iter()
            .any(|candidate| Arc::ptr_eq(candidate, &logger.data))
        {
            registry
                .attached_trace_loggers
                .push(Arc::clone(&logger.data));
        }

        *attached_logger = Some(Arc::downgrade(&self.inner));
        Ok(())
    }

    fn is_registered_trace_channel(&self, trace_namespace: &str, channel: &str) -> bool {
        if !is_selector_identifier(trace_namespace) || !is_valid_channel_path(channel) {
            return false;
        }

        self.inner
            .registry
            .lock()
            .ok()
            .and_then(|registry| {
                registry
                    .channels_by_namespace
                    .get(trace_namespace)
                    .map(|channels| channels.iter().any(|existing| existing == channel))
            })
            .unwrap_or(false)
    }

    fn is_trace_channel_enabled(&self, trace_namespace: &str, channel: &str) -> bool {
        if !self.is_registered_trace_channel(trace_namespace, channel) {
            return false;
        }
        let key = make_qualified_channel_key(trace_namespace, channel);
        self.inner
            .enabled_channel_keys
            .lock()
            .ok()
            .map(|enabled| enabled.contains(&key))
            .unwrap_or(false)
    }

    fn resolve_selectors_to_channel_keys(
        &self,
        selectors: &[Selector],
    ) -> TraceResult<SelectorResolution> {
        let registry = self
            .inner
            .registry
            .lock()
            .map_err(|_| TraceError::new("ktrace registry mutex is poisoned"))?;
        let mut seen = HashSet::new();
        let mut matched = vec![false; selectors.len()];
        let mut result = SelectorResolution::default();

        for (trace_namespace, channels) in &registry.channels_by_namespace {
            for channel in channels {
                for (index, selector) in selectors.iter().enumerate() {
                    if !matches_selector(selector, trace_namespace, channel) {
                        continue;
                    }
                    matched[index] = true;
                    let key = make_qualified_channel_key(trace_namespace, channel);
                    if seen.insert(key.clone()) {
                        result.channel_keys.push(key);
                    }
                }
            }
        }

        let mut unmatched_seen = HashSet::new();
        for (index, selector) in selectors.iter().enumerate() {
            if matched[index] {
                continue;
            }
            let selector_text = format_selector(selector);
            if unmatched_seen.insert(selector_text.clone()) {
                result.unmatched_selectors.push(selector_text);
            }
        }
        Ok(result)
    }

    fn emit_warning(&self, local_namespace: &str, message: String) -> TraceResult<()> {
        let namespace_label = if is_selector_identifier(local_namespace) {
            local_namespace
        } else {
            "ktrace"
        };
        let prefix = self.build_log_prefix(
            namespace_label,
            Severity::Warning,
            SourceLocation::new("", 0, ""),
        )?;
        emit_line(&self.inner, &prefix, &message)
    }

    pub fn enable_channel(
        &self,
        qualified_channel: impl AsRef<str>,
        local_namespace: impl AsRef<str>,
    ) -> TraceResult<()> {
        let (trace_namespace, channel) =
            parse_exact_selector(qualified_channel.as_ref(), local_namespace.as_ref())?;
        if !self.is_registered_trace_channel(&trace_namespace, &channel) {
            self.emit_warning(
                local_namespace.as_ref(),
                format!(
                    "enable ignored channel '{}' because it is not registered",
                    make_qualified_channel_key(&trace_namespace, &channel)
                ),
            )?;
            return Ok(());
        }

        let key = make_qualified_channel_key(&trace_namespace, &channel);
        self.inner
            .enabled_channel_keys
            .lock()
            .map_err(|_| TraceError::new("ktrace enablement mutex is poisoned"))?
            .insert(key);
        Ok(())
    }

    pub fn enable_channels(
        &self,
        selectors_csv: impl AsRef<str>,
        local_namespace: impl AsRef<str>,
    ) -> TraceResult<()> {
        let selector_text = trim_whitespace(selectors_csv.as_ref());
        if selector_text.is_empty() {
            return Err(TraceError::new(
                "EnableChannels requires one or more selectors",
            ));
        }

        let selectors = parse_selector_list(&selector_text, local_namespace.as_ref())?;
        let resolution = self.resolve_selectors_to_channel_keys(&selectors)?;
        {
            let mut enabled = self
                .inner
                .enabled_channel_keys
                .lock()
                .map_err(|_| TraceError::new("ktrace enablement mutex is poisoned"))?;
            for key in resolution.channel_keys {
                enabled.insert(key);
            }
        }
        for selector in resolution.unmatched_selectors {
            self.emit_warning(
                local_namespace.as_ref(),
                format!(
                    "enable ignored channel selector '{}' because it matched no registered channels",
                    selector
                ),
            )?;
        }
        Ok(())
    }

    pub fn should_trace_channel(
        &self,
        qualified_channel: impl AsRef<str>,
        local_namespace: impl AsRef<str>,
    ) -> bool {
        parse_exact_selector(qualified_channel.as_ref(), local_namespace.as_ref())
            .ok()
            .map(|(trace_namespace, channel)| {
                self.is_trace_channel_enabled(&trace_namespace, &channel)
            })
            .unwrap_or(false)
    }

    pub fn disable_channel(
        &self,
        qualified_channel: impl AsRef<str>,
        local_namespace: impl AsRef<str>,
    ) -> TraceResult<()> {
        let (trace_namespace, channel) =
            parse_exact_selector(qualified_channel.as_ref(), local_namespace.as_ref())?;
        if !self.is_registered_trace_channel(&trace_namespace, &channel) {
            self.emit_warning(
                local_namespace.as_ref(),
                format!(
                    "disable ignored channel '{}' because it is not registered",
                    make_qualified_channel_key(&trace_namespace, &channel)
                ),
            )?;
            return Ok(());
        }

        let key = make_qualified_channel_key(&trace_namespace, &channel);
        self.inner
            .enabled_channel_keys
            .lock()
            .map_err(|_| TraceError::new("ktrace enablement mutex is poisoned"))?
            .remove(&key);
        Ok(())
    }

    pub fn disable_channels(
        &self,
        selectors_csv: impl AsRef<str>,
        local_namespace: impl AsRef<str>,
    ) -> TraceResult<()> {
        let selector_text = trim_whitespace(selectors_csv.as_ref());
        if selector_text.is_empty() {
            return Err(TraceError::new(
                "DisableChannels requires one or more selectors",
            ));
        }

        let selectors = parse_selector_list(&selector_text, local_namespace.as_ref())?;
        let resolution = self.resolve_selectors_to_channel_keys(&selectors)?;
        {
            let mut enabled = self
                .inner
                .enabled_channel_keys
                .lock()
                .map_err(|_| TraceError::new("ktrace enablement mutex is poisoned"))?;
            for key in resolution.channel_keys {
                enabled.remove(&key);
            }
        }
        for selector in resolution.unmatched_selectors {
            self.emit_warning(
                local_namespace.as_ref(),
                format!(
                    "disable ignored channel selector '{}' because it matched no registered channels",
                    selector
                ),
            )?;
        }
        Ok(())
    }

    pub fn set_output_options(&self, options: OutputOptions) -> TraceResult<()> {
        let mut output_options = self
            .inner
            .output_options
            .lock()
            .map_err(|_| TraceError::new("ktrace output options mutex is poisoned"))?;
        *output_options = OutputOptions {
            filenames: options.filenames,
            line_numbers: options.filenames && options.line_numbers,
            function_names: options.filenames && options.function_names,
            timestamps: options.timestamps,
        };
        Ok(())
    }

    pub fn get_output_options(&self) -> TraceResult<OutputOptions> {
        self.inner
            .output_options
            .lock()
            .map(|options| *options)
            .map_err(|_| TraceError::new("ktrace output options mutex is poisoned"))
    }

    pub fn get_namespaces(&self) -> TraceResult<Vec<String>> {
        self.inner
            .registry
            .lock()
            .map(|registry| registry.namespaces.iter().cloned().collect())
            .map_err(|_| TraceError::new("ktrace registry mutex is poisoned"))
    }

    pub fn get_channels(&self, trace_namespace: impl AsRef<str>) -> TraceResult<Vec<String>> {
        let trace_namespace = trim_whitespace(trace_namespace.as_ref());
        if !is_selector_identifier(&trace_namespace) {
            return Err(TraceError::new(format!(
                "invalid trace namespace '{trace_namespace}'"
            )));
        }

        let mut channels = self
            .inner
            .registry
            .lock()
            .map_err(|_| TraceError::new("ktrace registry mutex is poisoned"))?
            .channels_by_namespace
            .get(&trace_namespace)
            .cloned()
            .unwrap_or_default();
        channels.sort();
        Ok(channels)
    }

    fn build_trace_prefix(
        &self,
        trace_namespace: &str,
        channel: &str,
        location: SourceLocation,
    ) -> TraceResult<String> {
        let output_options = self.get_output_options()?;
        let mut parts = Vec::new();
        if !trace_namespace.is_empty() {
            parts.push(format!("[{trace_namespace}]"));
        }
        if output_options.timestamps {
            parts.push(timestamp_label());
        }
        parts.push(format!("[{channel}]"));

        if output_options.filenames {
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
            parts.push(source);
        }

        Ok(parts.join(" "))
    }

    fn build_log_prefix(
        &self,
        trace_namespace: &str,
        severity: Severity,
        location: SourceLocation,
    ) -> TraceResult<String> {
        let output_options = self.get_output_options()?;
        let mut parts = Vec::new();
        if !trace_namespace.is_empty() {
            parts.push(format!("[{trace_namespace}]"));
        }
        if output_options.timestamps {
            parts.push(timestamp_label());
        }
        parts.push(format!(
            "[{}]",
            match severity {
                Severity::Info => "info",
                Severity::Warning => "warning",
                Severity::Error => "error",
            }
        ));

        if output_options.filenames && !location.file.is_empty() {
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
            parts.push(source);
        }

        Ok(parts.join(" "))
    }

    pub fn make_inline_parser(
        &self,
        local_trace_logger: TraceLogger,
        trace_root: impl AsRef<str>,
    ) -> Result<kcli::InlineParser, kcli::ConfigError> {
        let logger = self.clone();
        let local_namespace = local_trace_logger.namespace().to_string();

        let mut parser = kcli::InlineParser::new("trace")?;
        if !trace_root.as_ref().trim().is_empty() {
            parser.set_root(trace_root.as_ref())?;
        }

        {
            let logger = logger.clone();
            let local_namespace = local_namespace.clone();
            parser.set_root_value_handler_with_help(
                move |_context, value| {
                    logger
                        .enable_channels(value, &local_namespace)
                        .map_err(|error| error.to_string())
                },
                "<channels>",
                "Trace selected channels.",
            )?;
        }

        parser.set_flag_handler(
            "-examples",
            |context| {
                let option_root = format!("--{}", context.root);
                println!();
                println!("General trace selector pattern:");
                println!(
                    "  {} <namespace>.<channel>[.<subchannel>[.<subchannel>]]",
                    option_root
                );
                println!();
                println!("Trace selector examples:");
                println!("  {} '.abc'", option_root);
                println!("  {} 'otherapp.channel'", option_root);
                println!("  {} '*.*'", option_root);
                println!("  {} '*.*.*'", option_root);
                println!("  {} '*.*.*.*'", option_root);
                println!("  {} 'alpha.*'", option_root);
                println!("  {} 'alpha.*.*.*'", option_root);
                println!("  {} '*.net'", option_root);
                println!("  {} '*.{{net,io}}'", option_root);
                println!();
                Ok(())
            },
            "Show selector examples.",
        )?;

        {
            let logger = logger.clone();
            parser.set_flag_handler(
                "-namespaces",
                move |_context| {
                    let namespaces = logger.get_namespaces().map_err(|error| error.to_string())?;
                    if namespaces.is_empty() {
                        println!("No trace namespaces defined.");
                        println!();
                        return Ok(());
                    }
                    println!();
                    println!("Available trace namespaces:");
                    for trace_namespace in namespaces {
                        println!("  {trace_namespace}");
                    }
                    println!();
                    Ok(())
                },
                "Show initialized trace namespaces.",
            )?;
        }

        {
            let logger = logger.clone();
            parser.set_flag_handler(
                "-channels",
                move |_context| {
                    let namespaces = logger.get_namespaces().map_err(|error| error.to_string())?;
                    let mut printed_any = false;
                    for trace_namespace in namespaces {
                        let channels = logger
                            .get_channels(&trace_namespace)
                            .map_err(|error| error.to_string())?;
                        for channel in channels {
                            if !printed_any {
                                println!();
                                println!("Available trace channels:");
                                printed_any = true;
                            }
                            println!("  {}.{}", trace_namespace, channel);
                        }
                    }
                    if !printed_any {
                        println!("No trace channels defined.");
                        println!();
                        return Ok(());
                    }
                    println!();
                    Ok(())
                },
                "Show initialized trace channels.",
            )?;
        }

        parser.set_flag_handler(
            "-colors",
            |_context| {
                println!();
                println!("Available trace colors:");
                for color_name in available_color_names() {
                    println!("  {color_name}");
                }
                println!();
                Ok(())
            },
            "Show available trace colors.",
        )?;

        {
            let logger = logger.clone();
            parser.set_flag_handler(
                "-files",
                move |_context| {
                    let mut options = logger
                        .get_output_options()
                        .map_err(|error| error.to_string())?;
                    options.filenames = true;
                    options.line_numbers = true;
                    logger
                        .set_output_options(options)
                        .map_err(|error| error.to_string())
                },
                "Include source file and line in trace output.",
            )?;
        }

        {
            let logger = logger.clone();
            parser.set_flag_handler(
                "-functions",
                move |_context| {
                    let mut options = logger
                        .get_output_options()
                        .map_err(|error| error.to_string())?;
                    options.filenames = true;
                    options.line_numbers = true;
                    options.function_names = true;
                    logger
                        .set_output_options(options)
                        .map_err(|error| error.to_string())
                },
                "Include function names in trace output.",
            )?;
        }

        {
            let logger = logger.clone();
            parser.set_flag_handler(
                "-timestamps",
                move |_context| {
                    let mut options = logger
                        .get_output_options()
                        .map_err(|error| error.to_string())?;
                    options.timestamps = true;
                    logger
                        .set_output_options(options)
                        .map_err(|error| error.to_string())
                },
                "Include timestamps in trace output.",
            )?;
        }

        Ok(parser)
    }
}

#[macro_export]
macro_rules! ktrace_trace {
    ($logger:expr, $channel:expr, $($arg:tt)*) => {
        $logger.trace_with_location(
            $channel,
            $crate::SourceLocation::new(file!(), line!(), module_path!()),
            format!($($arg)*),
        )
    };
}

#[macro_export]
macro_rules! ktrace_trace_changed {
    ($logger:expr, $channel:expr, $key:expr, $($arg:tt)*) => {
        $logger.trace_changed_with_location(
            $channel,
            ($key).to_string(),
            $crate::SourceLocation::new(file!(), line!(), module_path!()),
            format!($($arg)*),
        )
    };
}

#[macro_export]
macro_rules! ktrace_info {
    ($logger:expr, $($arg:tt)*) => {
        $logger.log_with_location(
            $crate::Severity::Info,
            $crate::SourceLocation::new(file!(), line!(), module_path!()),
            format!($($arg)*),
        )
    };
}

#[macro_export]
macro_rules! ktrace_warn {
    ($logger:expr, $($arg:tt)*) => {
        $logger.log_with_location(
            $crate::Severity::Warning,
            $crate::SourceLocation::new(file!(), line!(), module_path!()),
            format!($($arg)*),
        )
    };
}

#[macro_export]
macro_rules! ktrace_error {
    ($logger:expr, $($arg:tt)*) => {
        $logger.log_with_location(
            $crate::Severity::Error,
            $crate::SourceLocation::new(file!(), line!(), module_path!()),
            format!($($arg)*),
        )
    };
}
