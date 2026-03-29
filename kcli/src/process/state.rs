use crate::model::{FlagHandler, InlineParserData, PositionalHandler, ValueHandler};

#[derive(Default)]
pub(crate) struct ParseOutcome {
    pub ok: bool,
    pub error: Option<ParseFailure>,
}

impl ParseOutcome {
    pub fn new() -> Self {
        Self {
            ok: true,
            error: None,
        }
    }
}

pub(crate) enum ParseFailure {
    AliasPresetsValuesForFlag { alias: String, option: String },
    MissingRequiredValue { option: String },
    UnknownRootValue { option: String },
    UnknownOption { option: String },
    HandlerFailed { option: String, message: String },
}

impl ParseFailure {
    pub fn option(&self) -> &str {
        match self {
            Self::AliasPresetsValuesForFlag { alias, .. } => alias,
            Self::MissingRequiredValue { option }
            | Self::UnknownRootValue { option }
            | Self::UnknownOption { option }
            | Self::HandlerFailed { option, .. } => option,
        }
    }

    pub fn render_message(&self) -> String {
        match self {
            Self::AliasPresetsValuesForFlag { alias, option } => format!(
                "alias '{}' presets values for option '{}' which does not accept values",
                alias, option
            ),
            Self::MissingRequiredValue { option } => format!("option '{option}' requires a value"),
            Self::UnknownRootValue { option } => format!("unknown value for option '{option}'"),
            Self::UnknownOption { option } => format!("unknown option {option}"),
            Self::HandlerFailed { option, message } => {
                if option.is_empty() {
                    message.clone()
                } else {
                    format!("option '{option}': {message}")
                }
            }
        }
    }
}

pub(crate) struct CollectedValues {
    pub has_value: bool,
    pub parts: Vec<String>,
    pub last_index: usize,
}

impl CollectedValues {
    pub fn new(option_index: usize) -> Self {
        Self {
            has_value: false,
            parts: Vec::new(),
            last_index: option_index,
        }
    }
}

#[derive(Clone)]
pub(crate) enum Invocation {
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
pub(crate) enum InlineTokenKind {
    None,
    BareRoot,
    DashOption,
}

pub(crate) struct InlineTokenMatch<'a> {
    pub kind: InlineTokenKind,
    pub parser: Option<&'a InlineParserData>,
    pub suffix: String,
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
