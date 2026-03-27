use crate::model::{
    AliasBinding, CommandBinding, ConfigError, FlagHandler, InlineParserData, ParserData,
    PositionalHandler, ValueArity, ValueHandler,
};
use crate::normalize::{
    normalize_alias, normalize_alias_target_option, normalize_description,
    normalize_help_placeholder, normalize_inline_handler_option,
    normalize_inline_root_option, normalize_primary_handler_option,
};

fn make_flag_binding(
    handler: FlagHandler,
    description: &str,
) -> Result<CommandBinding, ConfigError> {
    Ok(CommandBinding {
        expects_value: false,
        flag_handler: Some(handler),
        value_handler: None,
        value_arity: ValueArity::Required,
        description: normalize_description(description)?,
    })
}

fn make_value_binding(
    handler: ValueHandler,
    description: &str,
    arity: ValueArity,
) -> Result<CommandBinding, ConfigError> {
    Ok(CommandBinding {
        expects_value: true,
        flag_handler: None,
        value_handler: Some(handler),
        value_arity: arity,
        description: normalize_description(description)?,
    })
}

fn upsert_command(
    commands: &mut Vec<(String, CommandBinding)>,
    command: String,
    binding: CommandBinding,
) {
    for entry in commands.iter_mut() {
        if entry.0 == command {
            entry.1 = binding;
            return;
        }
    }
    commands.push((command, binding));
}

pub(crate) fn set_inline_root(
    data: &mut InlineParserData,
    root: &str,
) -> Result<(), ConfigError> {
    data.root_name = normalize_inline_root_option(root)?;
    Ok(())
}

pub(crate) fn set_root_value_handler(
    data: &mut InlineParserData,
    handler: ValueHandler,
) -> Result<(), ConfigError> {
    data.root_value_handler = Some(handler);
    data.root_value_placeholder.clear();
    data.root_value_description.clear();
    Ok(())
}

pub(crate) fn set_root_value_handler_with_help(
    data: &mut InlineParserData,
    handler: ValueHandler,
    value_placeholder: &str,
    description: &str,
) -> Result<(), ConfigError> {
    data.root_value_handler = Some(handler);
    data.root_value_placeholder = normalize_help_placeholder(value_placeholder)?;
    data.root_value_description = normalize_description(description)?;
    Ok(())
}

pub(crate) fn set_inline_flag_handler(
    data: &mut InlineParserData,
    option: &str,
    handler: FlagHandler,
    description: &str,
) -> Result<(), ConfigError> {
    let command = normalize_inline_handler_option(option, &data.root_name)?;
    let binding = make_flag_binding(handler, description)?;
    upsert_command(&mut data.commands, command, binding);
    Ok(())
}

pub(crate) fn set_inline_value_handler(
    data: &mut InlineParserData,
    option: &str,
    handler: ValueHandler,
    description: &str,
) -> Result<(), ConfigError> {
    let command = normalize_inline_handler_option(option, &data.root_name)?;
    let binding = make_value_binding(handler, description, ValueArity::Required)?;
    upsert_command(&mut data.commands, command, binding);
    Ok(())
}

pub(crate) fn set_inline_optional_value_handler(
    data: &mut InlineParserData,
    option: &str,
    handler: ValueHandler,
    description: &str,
) -> Result<(), ConfigError> {
    let command = normalize_inline_handler_option(option, &data.root_name)?;
    let binding = make_value_binding(handler, description, ValueArity::Optional)?;
    upsert_command(&mut data.commands, command, binding);
    Ok(())
}

pub(crate) fn set_alias<T: AsRef<str>>(
    data: &mut ParserData,
    alias: &str,
    target: &str,
    preset_tokens: &[T],
) -> Result<(), ConfigError> {
    let normalized_alias = normalize_alias(alias)?;
    let normalized_target = normalize_alias_target_option(target)?;
    let binding = AliasBinding {
        alias: normalized_alias.clone(),
        target_token: normalized_target,
        preset_tokens: preset_tokens
            .iter()
            .map(|token| token.as_ref().to_string())
            .collect(),
    };

    for existing in data.aliases.iter_mut() {
        if existing.alias == normalized_alias {
            *existing = binding;
            return Ok(());
        }
    }

    data.aliases.push(binding);
    Ok(())
}

pub(crate) fn set_primary_flag_handler(
    data: &mut ParserData,
    option: &str,
    handler: FlagHandler,
    description: &str,
) -> Result<(), ConfigError> {
    let command = normalize_primary_handler_option(option)?;
    let binding = make_flag_binding(handler, description)?;
    upsert_command(&mut data.commands, command, binding);
    Ok(())
}

pub(crate) fn set_primary_value_handler(
    data: &mut ParserData,
    option: &str,
    handler: ValueHandler,
    description: &str,
) -> Result<(), ConfigError> {
    let command = normalize_primary_handler_option(option)?;
    let binding = make_value_binding(handler, description, ValueArity::Required)?;
    upsert_command(&mut data.commands, command, binding);
    Ok(())
}

pub(crate) fn set_primary_optional_value_handler(
    data: &mut ParserData,
    option: &str,
    handler: ValueHandler,
    description: &str,
) -> Result<(), ConfigError> {
    let command = normalize_primary_handler_option(option)?;
    let binding = make_value_binding(handler, description, ValueArity::Optional)?;
    upsert_command(&mut data.commands, command, binding);
    Ok(())
}

pub(crate) fn set_positional_handler(
    data: &mut ParserData,
    handler: PositionalHandler,
) -> Result<(), ConfigError> {
    data.positional_handler = Some(handler);
    Ok(())
}

pub(crate) fn add_inline_parser(
    data: &mut ParserData,
    parser: InlineParserData,
) -> Result<(), ConfigError> {
    for existing in &data.inline_parsers {
        if existing.root_name == parser.root_name {
            return Err(ConfigError::new(format!(
                "kcli inline parser root '--{}' is already registered",
                parser.root_name
            )));
        }
    }

    data.inline_parsers.push(parser);
    Ok(())
}

