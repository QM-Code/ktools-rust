mod cli;
mod colors;
mod format;
mod registry;
mod selectors;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex, Weak};

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
