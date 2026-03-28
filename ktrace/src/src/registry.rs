use std::collections::HashSet;
use std::sync::{Arc, Weak};

use crate::format::emit_line;
use crate::selectors::{
    format_selector, is_selector_identifier, is_valid_channel_path, make_qualified_channel_key,
    matches_selector, merge_color, parse_exact_selector, parse_selector_list, trim_whitespace,
};

use super::{
    ChannelSpec, Logger, LoggerInner, OutputOptions, RegistryState, Selector, SelectorResolution,
    Severity, SourceLocation, TraceError, TraceLogger, TraceLoggerData, TraceResult, DEFAULT_COLOR,
};

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
                channels: std::sync::Mutex::new(Vec::new()),
                attached_logger: std::sync::Mutex::new(None),
                changed_keys: std::sync::Mutex::new(std::collections::HashMap::new()),
            }),
        })
    }

    pub fn add_channel(&self, channel: impl AsRef<str>, color: super::ColorId) -> TraceResult<()> {
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
                enabled_channel_keys: std::sync::Mutex::new(HashSet::new()),
                output_options: std::sync::Mutex::new(OutputOptions::default()),
                registry: std::sync::Mutex::new(RegistryState::default()),
                output_lock: std::sync::Mutex::new(()),
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

    fn set_exact_channel_state(
        &self,
        qualified_channel: &str,
        local_namespace: &str,
        enabled: bool,
    ) -> TraceResult<()> {
        let (trace_namespace, channel) = parse_exact_selector(qualified_channel, local_namespace)?;
        if !self.is_registered_trace_channel(&trace_namespace, &channel) {
            let action = if enabled { "enable" } else { "disable" };
            self.emit_warning(
                local_namespace,
                format!(
                    "{action} ignored channel '{}' because it is not registered",
                    make_qualified_channel_key(&trace_namespace, &channel)
                ),
            )?;
            return Ok(());
        }

        let key = make_qualified_channel_key(&trace_namespace, &channel);
        let mut enabled_keys = self
            .inner
            .enabled_channel_keys
            .lock()
            .map_err(|_| TraceError::new("ktrace enablement mutex is poisoned"))?;
        if enabled {
            enabled_keys.insert(key);
        } else {
            enabled_keys.remove(&key);
        }
        Ok(())
    }

    fn apply_channel_key_state(&self, channel_keys: Vec<String>, enabled: bool) -> TraceResult<()> {
        let mut enabled_keys = self
            .inner
            .enabled_channel_keys
            .lock()
            .map_err(|_| TraceError::new("ktrace enablement mutex is poisoned"))?;
        for key in channel_keys {
            if enabled {
                enabled_keys.insert(key);
            } else {
                enabled_keys.remove(&key);
            }
        }
        Ok(())
    }

    fn set_selector_list_state(
        &self,
        selectors_csv: &str,
        local_namespace: &str,
        enabled: bool,
    ) -> TraceResult<()> {
        let selector_text = trim_whitespace(selectors_csv);
        if selector_text.is_empty() {
            return Err(TraceError::new(if enabled {
                "EnableChannels requires one or more selectors"
            } else {
                "DisableChannels requires one or more selectors"
            }));
        }

        let selectors = parse_selector_list(&selector_text, local_namespace)?;
        let resolution = self.resolve_selectors_to_channel_keys(&selectors)?;
        self.apply_channel_key_state(resolution.channel_keys, enabled)?;

        for selector in resolution.unmatched_selectors {
            let action = if enabled { "enable" } else { "disable" };
            self.emit_warning(
                local_namespace,
                format!(
                    "{action} ignored channel selector '{}' because it matched no registered channels",
                    selector
                ),
            )?;
        }
        Ok(())
    }

    pub fn enable_channel(
        &self,
        qualified_channel: impl AsRef<str>,
        local_namespace: impl AsRef<str>,
    ) -> TraceResult<()> {
        self.set_exact_channel_state(qualified_channel.as_ref(), local_namespace.as_ref(), true)
    }

    pub fn enable_channels(
        &self,
        selectors_csv: impl AsRef<str>,
        local_namespace: impl AsRef<str>,
    ) -> TraceResult<()> {
        self.set_selector_list_state(selectors_csv.as_ref(), local_namespace.as_ref(), true)
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
        self.set_exact_channel_state(qualified_channel.as_ref(), local_namespace.as_ref(), false)
    }

    pub fn disable_channels(
        &self,
        selectors_csv: impl AsRef<str>,
        local_namespace: impl AsRef<str>,
    ) -> TraceResult<()> {
        self.set_selector_list_state(selectors_csv.as_ref(), local_namespace.as_ref(), false)
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
}
