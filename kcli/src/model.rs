use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HandlerContext {
    pub root: String,
    pub option: String,
    pub command: String,
    pub value_tokens: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliError {
    option: String,
    message: String,
}

impl CliError {
    pub fn new(option: impl Into<String>, message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            option: option.into(),
            message: if message.is_empty() {
                "kcli parse failed".to_string()
            } else {
                message
            },
        }
    }

    pub fn option(&self) -> &str {
        &self.option
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for CliError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigError {
    message: String,
}

impl ConfigError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ConfigError {}

pub type HandlerResult = Result<(), String>;

pub(crate) type FlagHandler = Arc<dyn Fn(&HandlerContext) -> HandlerResult + 'static>;
pub(crate) type ValueHandler = Arc<dyn Fn(&HandlerContext, &str) -> HandlerResult + 'static>;
pub(crate) type PositionalHandler = Arc<dyn Fn(&HandlerContext) -> HandlerResult + 'static>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ValueArity {
    Required,
    Optional,
}

impl Default for ValueArity {
    fn default() -> Self {
        Self::Required
    }
}

#[derive(Clone, Default)]
pub(crate) struct CommandBinding {
    pub expects_value: bool,
    pub flag_handler: Option<FlagHandler>,
    pub value_handler: Option<ValueHandler>,
    pub value_arity: ValueArity,
    pub description: String,
}

#[derive(Clone, Default)]
pub(crate) struct AliasBinding {
    pub alias: String,
    pub target_token: String,
    pub preset_tokens: Vec<String>,
}

#[derive(Clone, Default)]
pub(crate) struct AliasTable {
    entries: HashMap<String, AliasBinding>,
}

impl AliasTable {
    pub fn insert(&mut self, alias: String, binding: AliasBinding) {
        self.entries.insert(alias, binding);
    }

    pub fn get(&self, alias: &str) -> Option<&AliasBinding> {
        self.entries.get(alias)
    }
}

#[derive(Clone, Default)]
pub(crate) struct CommandTable {
    order: Vec<String>,
    entries: HashMap<String, CommandBinding>,
}

impl CommandTable {
    pub fn insert_or_replace(&mut self, command: String, binding: CommandBinding) {
        if !self.entries.contains_key(&command) {
            self.order.push(command.clone());
        }
        self.entries.insert(command, binding);
    }

    pub fn get(&self, command: &str) -> Option<&CommandBinding> {
        self.entries.get(command)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &CommandBinding)> {
        self.order.iter().filter_map(|command| {
            self.entries
                .get(command)
                .map(|binding| (command.as_str(), binding))
        })
    }
}

#[derive(Clone, Default)]
pub(crate) struct InlineParserData {
    pub root_name: String,
    pub root_value_handler: Option<ValueHandler>,
    pub root_value_placeholder: String,
    pub root_value_description: String,
    pub commands: CommandTable,
}

#[derive(Clone, Default)]
pub(crate) struct InlineParserTable {
    order: Vec<String>,
    entries: HashMap<String, InlineParserData>,
}

impl InlineParserTable {
    pub fn contains(&self, root_name: &str) -> bool {
        self.entries.contains_key(root_name)
    }

    pub fn insert(&mut self, parser: InlineParserData) {
        if !self.entries.contains_key(&parser.root_name) {
            self.order.push(parser.root_name.clone());
        }
        self.entries.insert(parser.root_name.clone(), parser);
    }

    pub fn iter(&self) -> impl Iterator<Item = &InlineParserData> {
        self.order
            .iter()
            .filter_map(|root_name| self.entries.get(root_name))
    }
}

#[derive(Clone, Default)]
pub(crate) struct ParserData {
    pub positional_handler: Option<PositionalHandler>,
    pub aliases: AliasTable,
    pub commands: CommandTable,
    pub inline_parsers: InlineParserTable,
}
