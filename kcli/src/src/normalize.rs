use std::io::{self, IsTerminal, Write};

use crate::model::ConfigError;

pub(crate) fn report_cli_error_and_exit(message: &str) -> ! {
    let mut stderr = io::stderr();
    if stderr.is_terminal() {
        let _ = writeln!(
            stderr,
            "[\x1b[31merror\x1b[0m] [\x1b[94mcli\x1b[0m] {message}"
        );
    } else {
        let _ = writeln!(stderr, "[error] [cli] {message}");
    }
    let _ = stderr.flush();
    std::process::exit(2);
}

pub(crate) fn starts_with(value: &str, prefix: &str) -> bool {
    value.starts_with(prefix)
}

fn trim_whitespace(value: &str) -> &str {
    value.trim()
}

fn contains_whitespace(value: &str) -> bool {
    value.chars().any(char::is_whitespace)
}

pub(crate) fn normalize_root_name(raw_root: &str) -> Result<String, ConfigError> {
    let root = trim_whitespace(raw_root);
    if root.is_empty() {
        return Err(ConfigError::new("kcli root must not be empty"));
    }
    if root.starts_with('-') {
        return Err(ConfigError::new("kcli root must not begin with '-'"));
    }
    if contains_whitespace(root) {
        return Err(ConfigError::new("kcli root is invalid"));
    }
    Ok(root.to_string())
}

pub(crate) fn normalize_inline_root_option(raw_root: &str) -> Result<String, ConfigError> {
    let root = trim_whitespace(raw_root);
    if root.is_empty() {
        return Err(ConfigError::new("kcli root must not be empty"));
    }
    if let Some(stripped) = root.strip_prefix("--") {
        return normalize_root_name(stripped);
    }
    if root.starts_with('-') {
        return Err(ConfigError::new("kcli root must use '--root' or 'root'"));
    }
    normalize_root_name(root)
}

pub(crate) fn normalize_inline_handler_option(
    raw_option: &str,
    root_name: &str,
) -> Result<String, ConfigError> {
    let option = trim_whitespace(raw_option);
    if option.is_empty() {
        return Err(ConfigError::new(
            "kcli inline handler option must not be empty",
        ));
    }

    let normalized = if let Some(stripped) = option.strip_prefix("--") {
        let full_prefix = format!("{root_name}-");
        if !starts_with(stripped, &full_prefix) {
            return Err(ConfigError::new(format!(
                "kcli inline handler option must use '-name' or '--{root_name}-name'"
            )));
        }
        stripped[full_prefix.len()..].to_string()
    } else if let Some(stripped) = option.strip_prefix('-') {
        stripped.to_string()
    } else {
        return Err(ConfigError::new(format!(
            "kcli inline handler option must use '-name' or '--{root_name}-name'"
        )));
    };

    if normalized.is_empty() {
        return Err(ConfigError::new("kcli command must not be empty"));
    }
    if normalized.starts_with('-') {
        return Err(ConfigError::new("kcli command must not start with '-'"));
    }
    if contains_whitespace(&normalized) {
        return Err(ConfigError::new(
            "kcli command must not contain whitespace",
        ));
    }
    Ok(normalized)
}

pub(crate) fn normalize_primary_handler_option(
    raw_option: &str,
) -> Result<String, ConfigError> {
    let option = trim_whitespace(raw_option);
    if option.is_empty() {
        return Err(ConfigError::new(
            "kcli end-user handler option must not be empty",
        ));
    }

    let normalized = if let Some(stripped) = option.strip_prefix("--") {
        stripped.to_string()
    } else if option.starts_with('-') {
        return Err(ConfigError::new(
            "kcli end-user handler option must use '--name' or 'name'",
        ));
    } else {
        option.to_string()
    };

    if normalized.is_empty() {
        return Err(ConfigError::new("kcli command must not be empty"));
    }
    if normalized.starts_with('-') {
        return Err(ConfigError::new("kcli command must not start with '-'"));
    }
    if contains_whitespace(&normalized) {
        return Err(ConfigError::new(
            "kcli command must not contain whitespace",
        ));
    }
    Ok(normalized)
}

pub(crate) fn normalize_alias(raw_alias: &str) -> Result<String, ConfigError> {
    let alias = trim_whitespace(raw_alias);
    if alias.len() < 2 || !alias.starts_with('-') || alias.starts_with("--") || contains_whitespace(alias)
    {
        return Err(ConfigError::new(
            "kcli alias must use single-dash form, e.g. '-v'",
        ));
    }
    Ok(alias.to_string())
}

pub(crate) fn normalize_alias_target_option(
    raw_target: &str,
) -> Result<String, ConfigError> {
    let target = trim_whitespace(raw_target);
    if target.len() < 3 || !target.starts_with("--") || contains_whitespace(target) {
        return Err(ConfigError::new(
            "kcli alias target must use double-dash form, e.g. '--verbose'",
        ));
    }
    if target.as_bytes()[2] == b'-' {
        return Err(ConfigError::new(
            "kcli alias target must use double-dash form, e.g. '--verbose'",
        ));
    }
    Ok(target.to_string())
}

pub(crate) fn normalize_help_placeholder(
    raw_placeholder: &str,
) -> Result<String, ConfigError> {
    let placeholder = trim_whitespace(raw_placeholder);
    if placeholder.is_empty() {
        return Err(ConfigError::new(
            "kcli help placeholder must not be empty",
        ));
    }
    Ok(placeholder.to_string())
}

pub(crate) fn normalize_description(raw_description: &str) -> Result<String, ConfigError> {
    let description = trim_whitespace(raw_description);
    if description.is_empty() {
        return Err(ConfigError::new(
            "kcli command description must not be empty",
        ));
    }
    Ok(description.to_string())
}

