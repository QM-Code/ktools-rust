mod backend;
pub mod demo;
mod model;
mod normalize;
mod process;

use std::sync::Arc;

use backend::{
    add_inline_parser, set_alias, set_inline_flag_handler, set_inline_optional_value_handler,
    set_inline_root, set_inline_value_handler, set_positional_handler,
    set_primary_flag_handler, set_primary_optional_value_handler,
    set_primary_value_handler, set_root_value_handler,
    set_root_value_handler_with_help,
};
use model::{InlineParserData, ParserData, PositionalHandler};
use normalize::report_cli_error_and_exit;
use process::parse_tokens;

pub use model::{CliError, ConfigError, HandlerContext, HandlerResult};

pub struct InlineParser {
    data: InlineParserData,
}

impl InlineParser {
    pub fn new(root: impl AsRef<str>) -> Result<Self, ConfigError> {
        let mut data = InlineParserData::default();
        set_inline_root(&mut data, root.as_ref())?;
        Ok(Self { data })
    }

    pub fn set_root(&mut self, root: impl AsRef<str>) -> Result<(), ConfigError> {
        set_inline_root(&mut self.data, root.as_ref())
    }

    pub fn set_root_value_handler<F>(&mut self, handler: F) -> Result<(), ConfigError>
    where
        F: Fn(&HandlerContext, &str) -> HandlerResult + 'static,
    {
        set_root_value_handler(&mut self.data, Arc::new(handler))
    }

    pub fn set_root_value_handler_with_help<F>(
        &mut self,
        handler: F,
        value_placeholder: impl AsRef<str>,
        description: impl AsRef<str>,
    ) -> Result<(), ConfigError>
    where
        F: Fn(&HandlerContext, &str) -> HandlerResult + 'static,
    {
        set_root_value_handler_with_help(
            &mut self.data,
            Arc::new(handler),
            value_placeholder.as_ref(),
            description.as_ref(),
        )
    }

    pub fn set_flag_handler<F>(
        &mut self,
        option: impl AsRef<str>,
        handler: F,
        description: impl AsRef<str>,
    ) -> Result<(), ConfigError>
    where
        F: Fn(&HandlerContext) -> HandlerResult + 'static,
    {
        set_inline_flag_handler(
            &mut self.data,
            option.as_ref(),
            Arc::new(handler),
            description.as_ref(),
        )
    }

    pub fn set_value_handler<F>(
        &mut self,
        option: impl AsRef<str>,
        handler: F,
        description: impl AsRef<str>,
    ) -> Result<(), ConfigError>
    where
        F: Fn(&HandlerContext, &str) -> HandlerResult + 'static,
    {
        set_inline_value_handler(
            &mut self.data,
            option.as_ref(),
            Arc::new(handler),
            description.as_ref(),
        )
    }

    pub fn set_optional_value_handler<F>(
        &mut self,
        option: impl AsRef<str>,
        handler: F,
        description: impl AsRef<str>,
    ) -> Result<(), ConfigError>
    where
        F: Fn(&HandlerContext, &str) -> HandlerResult + 'static,
    {
        set_inline_optional_value_handler(
            &mut self.data,
            option.as_ref(),
            Arc::new(handler),
            description.as_ref(),
        )
    }
}

pub struct Parser {
    data: ParserData,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            data: ParserData::default(),
        }
    }

    pub fn add_alias(
        &mut self,
        alias: impl AsRef<str>,
        target: impl AsRef<str>,
        preset_tokens: &[impl AsRef<str>],
    ) -> Result<(), ConfigError> {
        set_alias(
            &mut self.data,
            alias.as_ref(),
            target.as_ref(),
            preset_tokens,
        )
    }

    pub fn set_flag_handler<F>(
        &mut self,
        option: impl AsRef<str>,
        handler: F,
        description: impl AsRef<str>,
    ) -> Result<(), ConfigError>
    where
        F: Fn(&HandlerContext) -> HandlerResult + 'static,
    {
        set_primary_flag_handler(
            &mut self.data,
            option.as_ref(),
            Arc::new(handler),
            description.as_ref(),
        )
    }

    pub fn set_value_handler<F>(
        &mut self,
        option: impl AsRef<str>,
        handler: F,
        description: impl AsRef<str>,
    ) -> Result<(), ConfigError>
    where
        F: Fn(&HandlerContext, &str) -> HandlerResult + 'static,
    {
        set_primary_value_handler(
            &mut self.data,
            option.as_ref(),
            Arc::new(handler),
            description.as_ref(),
        )
    }

    pub fn set_optional_value_handler<F>(
        &mut self,
        option: impl AsRef<str>,
        handler: F,
        description: impl AsRef<str>,
    ) -> Result<(), ConfigError>
    where
        F: Fn(&HandlerContext, &str) -> HandlerResult + 'static,
    {
        set_primary_optional_value_handler(
            &mut self.data,
            option.as_ref(),
            Arc::new(handler),
            description.as_ref(),
        )
    }

    pub fn set_positional_handler<F>(&mut self, handler: F) -> Result<(), ConfigError>
    where
        F: Fn(&HandlerContext) -> HandlerResult + 'static,
    {
        let handler: PositionalHandler = Arc::new(handler);
        set_positional_handler(&mut self.data, handler)
    }

    pub fn add_inline_parser(&mut self, parser: InlineParser) -> Result<(), ConfigError> {
        add_inline_parser(&mut self.data, parser.data)
    }

    pub fn parse<T: AsRef<str>>(&self, argv: &[T]) -> Result<(), CliError> {
        let tokens = argv
            .iter()
            .map(|value| value.as_ref().to_string())
            .collect::<Vec<_>>();
        parse_tokens(&self.data, &tokens)
    }

    pub fn parse_or_exit<T: AsRef<str>>(&self, argv: &[T]) {
        if let Err(error) = self.parse(argv) {
            report_cli_error_and_exit(&error.to_string());
        }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}
