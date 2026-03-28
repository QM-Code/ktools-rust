use crate::model::{
    AliasBinding, CliError, CommandBinding, FlagHandler, HandlerContext, InlineParserData,
    ParserData, PositionalHandler, ValueArity, ValueHandler,
};
use crate::normalize::starts_with;

#[derive(Default)]
struct ParseOutcome {
    ok: bool,
    error_option: String,
    error_message: String,
}

impl ParseOutcome {
    fn new() -> Self {
        Self {
            ok: true,
            error_option: String::new(),
            error_message: String::new(),
        }
    }
}

struct CollectedValues {
    has_value: bool,
    parts: Vec<String>,
    last_index: usize,
}

impl CollectedValues {
    fn new(option_index: usize) -> Self {
        Self {
            has_value: false,
            parts: Vec::new(),
            last_index: option_index,
        }
    }
}

#[derive(Clone)]
enum Invocation {
    Flag {
        root: String,
        option: String,
        command: String,
        handler: FlagHandler,
    },
    Value {
        root: String,
        option: String,
        command: String,
        value_tokens: Vec<String>,
        handler: ValueHandler,
    },
    Positional {
        value_tokens: Vec<String>,
        handler: PositionalHandler,
    },
    PrintHelp {
        root: String,
        help_rows: Vec<(String, String)>,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InlineTokenKind {
    None,
    BareRoot,
    DashOption,
}

struct InlineTokenMatch<'a> {
    kind: InlineTokenKind,
    parser: Option<&'a InlineParserData>,
    suffix: String,
}

impl<'a> Default for InlineTokenMatch<'a> {
    fn default() -> Self {
        Self {
            kind: InlineTokenKind::None,
            parser: None,
            suffix: String::new(),
        }
    }
}

fn is_collectable_follow_on_value_token(value: &str) -> bool {
    !value.starts_with('-')
}

fn join_with_spaces(parts: &[String]) -> String {
    parts.join(" ")
}

fn format_option_error_message(option: &str, message: &str) -> String {
    if option.is_empty() {
        return message.to_string();
    }
    format!("option '{option}': {message}")
}

fn report_error(result: &mut ParseOutcome, option: &str, message: impl Into<String>) {
    if result.ok {
        result.ok = false;
        result.error_option = option.to_string();
        result.error_message = message.into();
    }
}

fn collect_value_tokens(
    option_index: usize,
    tokens: &[String],
    consumed: &mut [bool],
    allow_option_like_first_value: bool,
) -> CollectedValues {
    let mut collected = CollectedValues::new(option_index);
    let first_value_index = option_index + 1;

    if first_value_index >= tokens.len() || consumed[first_value_index] {
        return collected;
    }

    let first = &tokens[first_value_index];
    if !allow_option_like_first_value && first.starts_with('-') {
        return collected;
    }

    collected.has_value = true;
    collected.parts.push(first.clone());
    consumed[first_value_index] = true;
    collected.last_index = first_value_index;

    if allow_option_like_first_value && first.starts_with('-') {
        return collected;
    }

    for scan in first_value_index + 1..tokens.len() {
        if consumed[scan] {
            continue;
        }

        let next = &tokens[scan];
        if !is_collectable_follow_on_value_token(next) {
            break;
        }

        collected.parts.push(next.clone());
        consumed[scan] = true;
        collected.last_index = scan;
    }

    collected
}

fn print_help(root: &str, help_rows: &[(String, String)]) {
    println!();
    println!("Available --{}-* options:", root);

    let max_lhs = help_rows
        .iter()
        .map(|(lhs, _)| lhs.len())
        .max()
        .unwrap_or(0);

    if help_rows.is_empty() {
        println!("  (no options registered)");
    } else {
        for (lhs, rhs) in help_rows {
            let padding = max_lhs.saturating_sub(lhs.len());
            println!("  {lhs}{}{}", " ".repeat(padding + 2), rhs);
        }
    }

    println!();
}

fn consume_index(consumed: &mut [bool], index: usize) {
    if index < consumed.len() && !consumed[index] {
        consumed[index] = true;
    }
}

fn find_command<'a>(
    commands: &'a [(String, CommandBinding)],
    command: &str,
) -> Option<&'a CommandBinding> {
    commands
        .iter()
        .find(|(existing_command, _)| existing_command == command)
        .map(|(_, binding)| binding)
}

fn find_alias_binding<'a>(data: &'a ParserData, token: &str) -> Option<&'a AliasBinding> {
    data.aliases.iter().find(|alias| alias.alias == token)
}

fn has_alias_preset_tokens(alias_binding: Option<&AliasBinding>) -> bool {
    alias_binding
        .map(|binding| !binding.preset_tokens.is_empty())
        .unwrap_or(false)
}

fn build_effective_value_tokens(
    alias_binding: Option<&AliasBinding>,
    collected_parts: &[String],
) -> Vec<String> {
    let mut merged = Vec::new();
    if let Some(binding) = alias_binding {
        merged.extend(binding.preset_tokens.iter().cloned());
    }
    merged.extend(collected_parts.iter().cloned());
    merged
}

fn build_help_rows(parser: &InlineParserData) -> Vec<(String, String)> {
    let prefix = format!("--{}-", parser.root_name);
    let mut rows = Vec::new();

    if parser.root_value_handler.is_some() && !parser.root_value_description.is_empty() {
        let mut lhs = format!("--{}", parser.root_name);
        if !parser.root_value_placeholder.is_empty() {
            lhs.push(' ');
            lhs.push_str(&parser.root_value_placeholder);
        }
        rows.push((lhs, parser.root_value_description.clone()));
    }

    for (command, binding) in &parser.commands {
        let mut lhs = format!("{prefix}{command}");
        if binding.expects_value {
            match binding.value_arity {
                ValueArity::Optional => lhs.push_str(" [value]"),
                ValueArity::Required => lhs.push_str(" <value>"),
            }
        }
        rows.push((lhs, binding.description.clone()));
    }

    rows
}

fn match_inline_token<'a>(data: &'a ParserData, arg: &str) -> InlineTokenMatch<'a> {
    for parser in &data.inline_parsers {
        let root_option = format!("--{}", parser.root_name);
        if arg == root_option {
            return InlineTokenMatch {
                kind: InlineTokenKind::BareRoot,
                parser: Some(parser),
                suffix: String::new(),
            };
        }

        let root_dash_prefix = format!("{root_option}-");
        if starts_with(arg, &root_dash_prefix) {
            return InlineTokenMatch {
                kind: InlineTokenKind::DashOption,
                parser: Some(parser),
                suffix: arg[root_dash_prefix.len()..].to_string(),
            };
        }
    }

    InlineTokenMatch::default()
}

fn schedule_invocation(
    binding: &CommandBinding,
    alias_binding: Option<&AliasBinding>,
    root: &str,
    command: &str,
    option_token: &str,
    index: usize,
    tokens: &[String],
    consumed: &mut [bool],
    invocations: &mut Vec<Invocation>,
    result: &mut ParseOutcome,
) -> usize {
    consume_index(consumed, index);

    if !binding.expects_value {
        if let Some(alias_binding) = alias_binding {
            if !alias_binding.preset_tokens.is_empty() {
                report_error(
                    result,
                    &alias_binding.alias,
                    format!(
                        "alias '{}' presets values for option '{}' which does not accept values",
                        alias_binding.alias, option_token
                    ),
                );
                return index;
            }
        }

        invocations.push(Invocation::Flag {
            root: root.to_string(),
            option: option_token.to_string(),
            command: command.to_string(),
            handler: binding
                .flag_handler
                .clone()
                .expect("flag binding must carry a flag handler"),
        });
        return index;
    }

    let collected = collect_value_tokens(
        index,
        tokens,
        consumed,
        binding.value_arity == ValueArity::Required,
    );

    if !collected.has_value
        && !has_alias_preset_tokens(alias_binding)
        && binding.value_arity == ValueArity::Required
    {
        report_error(
            result,
            option_token,
            format!("option '{option_token}' requires a value"),
        );
        return index;
    }

    let mut final_index = index;
    if collected.has_value {
        final_index = collected.last_index;
    }

    invocations.push(Invocation::Value {
        root: root.to_string(),
        option: option_token.to_string(),
        command: command.to_string(),
        value_tokens: build_effective_value_tokens(alias_binding, &collected.parts),
        handler: binding
            .value_handler
            .clone()
            .expect("value binding must carry a value handler"),
    });
    final_index
}

fn schedule_positionals(
    data: &ParserData,
    tokens: &[String],
    consumed: &mut [bool],
    invocations: &mut Vec<Invocation>,
) {
    let Some(handler) = data.positional_handler.clone() else {
        return;
    };

    if tokens.len() <= 1 {
        return;
    }

    let mut value_tokens = Vec::new();

    for index in 1..tokens.len() {
        if consumed[index] {
            continue;
        }

        let token = &tokens[index];
        if token.is_empty() || !token.starts_with('-') {
            consumed[index] = true;
            value_tokens.push(token.clone());
        }
    }

    if !value_tokens.is_empty() {
        invocations.push(Invocation::Positional {
            value_tokens,
            handler,
        });
    }
}

fn execute_invocations(invocations: &[Invocation], result: &mut ParseOutcome) {
    for invocation in invocations {
        if !result.ok {
            return;
        }

        let execution = match invocation {
            Invocation::Flag {
                root,
                option,
                command,
                handler,
            } => handler(&HandlerContext {
                root: root.clone(),
                option: option.clone(),
                command: command.clone(),
                value_tokens: Vec::new(),
            }),
            Invocation::Value {
                root,
                option,
                command,
                value_tokens,
                handler,
            } => {
                let context = HandlerContext {
                    root: root.clone(),
                    option: option.clone(),
                    command: command.clone(),
                    value_tokens: value_tokens.clone(),
                };
                handler(&context, &join_with_spaces(value_tokens))
            }
            Invocation::Positional {
                value_tokens,
                handler,
            } => handler(&HandlerContext {
                root: String::new(),
                option: String::new(),
                command: String::new(),
                value_tokens: value_tokens.clone(),
            }),
            Invocation::PrintHelp { root, help_rows } => {
                print_help(root, help_rows);
                Ok(())
            }
        };

        if let Err(message) = execution {
            let option = match invocation {
                Invocation::Flag { option, .. } => option.as_str(),
                Invocation::Value { option, .. } => option.as_str(),
                Invocation::Positional { .. } | Invocation::PrintHelp { .. } => "",
            };
            report_error(
                result,
                option,
                format_option_error_message(option, &message),
            );
        }
    }
}

fn plan_invocations(
    data: &ParserData,
    tokens: &[String],
    consumed: &mut [bool],
    result: &mut ParseOutcome,
) -> Vec<Invocation> {
    let mut invocations = Vec::new();
    let mut index = 1usize;
    while index < tokens.len() {
        if consumed[index] {
            index += 1;
            continue;
        }

        let arg = &tokens[index];
        if arg.is_empty() {
            index += 1;
            continue;
        }

        let alias_binding = if arg.starts_with('-') && !starts_with(arg, "--") {
            find_alias_binding(data, arg)
        } else {
            None
        };

        let effective_arg = alias_binding
            .map(|binding| binding.target_token.as_str())
            .unwrap_or(arg.as_str());

        if !effective_arg.starts_with('-') {
            index += 1;
            continue;
        }

        if effective_arg == "--" {
            index += 1;
            continue;
        }

        if starts_with(effective_arg, "--") {
            let inline_match = match_inline_token(data, effective_arg);
            match inline_match.kind {
                InlineTokenKind::BareRoot => {
                    let parser = inline_match.parser.expect("bare root must have parser");
                    consume_index(consumed, index);
                    let collected = collect_value_tokens(index, tokens, consumed, false);

                    if !collected.has_value && !has_alias_preset_tokens(alias_binding) {
                        invocations.push(Invocation::PrintHelp {
                            root: parser.root_name.clone(),
                            help_rows: build_help_rows(parser),
                        });
                    } else if parser.root_value_handler.is_none() {
                        report_error(
                            result,
                            effective_arg,
                            format!("unknown value for option '{effective_arg}'"),
                        );
                    } else {
                        invocations.push(Invocation::Value {
                            root: parser.root_name.clone(),
                            option: effective_arg.to_string(),
                            value_tokens: build_effective_value_tokens(
                                alias_binding,
                                &collected.parts,
                            ),
                            command: String::new(),
                            handler: parser
                                .root_value_handler
                                .clone()
                                .expect("root value handler must exist"),
                        });
                        if collected.has_value {
                            index = collected.last_index;
                        }
                    }
                }
                InlineTokenKind::DashOption => {
                    let parser = inline_match
                        .parser
                        .expect("dash inline option must have parser");
                    if !inline_match.suffix.is_empty() {
                        if let Some(binding) = find_command(&parser.commands, &inline_match.suffix)
                        {
                            index = schedule_invocation(
                                binding,
                                alias_binding,
                                &parser.root_name,
                                &inline_match.suffix,
                                effective_arg,
                                index,
                                tokens,
                                consumed,
                                &mut invocations,
                                result,
                            );
                        }
                    }
                }
                InlineTokenKind::None => {
                    let command = &effective_arg[2..];
                    if let Some(binding) = find_command(&data.commands, command) {
                        index = schedule_invocation(
                            binding,
                            alias_binding,
                            "",
                            command,
                            effective_arg,
                            index,
                            tokens,
                            consumed,
                            &mut invocations,
                            result,
                        );
                    }
                }
            }
        }

        if !result.ok {
            break;
        }

        index += 1;
    }

    if result.ok {
        schedule_positionals(data, tokens, consumed, &mut invocations);
    }
    invocations
}

fn report_unconsumed_option_tokens(
    tokens: &[String],
    consumed: &[bool],
    result: &mut ParseOutcome,
) {
    if !result.ok {
        return;
    }

    for index in 1..tokens.len() {
        if consumed[index] {
            continue;
        }

        let token = &tokens[index];
        if token.is_empty() {
            continue;
        }
        if token.starts_with('-') {
            report_error(result, token, format!("unknown option {token}"));
            break;
        }
    }
}

pub(crate) fn parse_tokens(data: &ParserData, argv: &[String]) -> Result<(), CliError> {
    if argv.is_empty() {
        return Ok(());
    }

    let tokens = argv.to_vec();
    let mut consumed = vec![false; tokens.len()];
    let mut result = ParseOutcome::new();
    let invocations = plan_invocations(data, &tokens, &mut consumed, &mut result);

    report_unconsumed_option_tokens(&tokens, &consumed, &mut result);
    execute_invocations(&invocations, &mut result);

    if result.ok {
        Ok(())
    } else {
        Err(CliError::new(result.error_option, result.error_message))
    }
}
