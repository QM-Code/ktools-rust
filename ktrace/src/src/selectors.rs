use super::{ColorId, Selector, TraceError, TraceResult, DEFAULT_COLOR};

pub(crate) fn trim_whitespace(value: &str) -> String {
    value.trim().to_string()
}

pub(crate) fn is_selector_identifier(token: &str) -> bool {
    !token.is_empty()
        && token
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

pub(crate) fn split_channel_path(channel: &str) -> Option<Vec<&str>> {
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

pub(crate) fn is_valid_channel_path(channel: &str) -> bool {
    split_channel_path(channel).is_some()
}

pub(crate) fn make_qualified_channel_key(trace_namespace: &str, channel: &str) -> String {
    if trace_namespace.is_empty() || channel.is_empty() {
        return String::new();
    }
    format!("{trace_namespace}.{channel}")
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

pub(crate) fn format_selector(selector: &Selector) -> String {
    let namespace_label = if selector.any_namespace {
        "*".to_string()
    } else {
        selector.trace_namespace.clone()
    };
    format!("{namespace_label}.{}", selector.channel_tokens.join("."))
}

pub(crate) fn parse_selector_list(list: &str, local_namespace: &str) -> TraceResult<Vec<Selector>> {
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

pub(crate) fn matches_selector(selector: &Selector, trace_namespace: &str, channel: &str) -> bool {
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

pub(crate) fn parse_exact_selector(
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

pub(crate) fn merge_color(
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
