use crate::model::{InlineParserData, ValueArity};

pub(crate) fn build_help_rows(parser: &InlineParserData) -> Vec<(String, String)> {
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

    for (command, binding) in parser.commands.iter() {
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

pub(crate) fn print_help(root: &str, help_rows: &[(String, String)]) {
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::backend::{
        set_inline_flag_handler, set_inline_root, set_inline_value_handler,
        set_root_value_handler_with_help,
    };
    use crate::model::{HandlerContext, HandlerResult, InlineParserData};

    use super::build_help_rows;

    fn noop_flag(_context: &HandlerContext) -> HandlerResult {
        Ok(())
    }

    fn noop_value(_context: &HandlerContext, _value: &str) -> HandlerResult {
        Ok(())
    }

    #[test]
    fn help_rows_include_registered_inline_options() -> Result<(), Box<dyn std::error::Error>> {
        let mut parser = InlineParserData::default();
        set_inline_root(&mut parser, "build")?;
        set_inline_flag_handler(
            &mut parser,
            "-flag",
            Arc::new(noop_flag),
            "Enable build flag.",
        )?;
        set_inline_value_handler(
            &mut parser,
            "-value",
            Arc::new(noop_value),
            "Set build value.",
        )?;

        let rows = build_help_rows(&parser);

        assert_eq!(
            rows,
            vec![
                (
                    "--build-flag".to_string(),
                    "Enable build flag.".to_string(),
                ),
                (
                    "--build-value <value>".to_string(),
                    "Set build value.".to_string(),
                ),
            ]
        );
        Ok(())
    }

    #[test]
    fn help_rows_include_root_value_help_when_present(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut parser = InlineParserData::default();
        set_inline_root(&mut parser, "build")?;
        set_root_value_handler_with_help(
            &mut parser,
            Arc::new(noop_value),
            "<selector>",
            "Select build targets.",
        )?;
        set_inline_flag_handler(
            &mut parser,
            "-flag",
            Arc::new(noop_flag),
            "Enable build flag.",
        )?;

        let rows = build_help_rows(&parser);

        assert_eq!(
            rows[0],
            (
                "--build <selector>".to_string(),
                "Select build targets.".to_string(),
            )
        );
        assert_eq!(
            rows[1],
            (
                "--build-flag".to_string(),
                "Enable build flag.".to_string(),
            )
        );
        Ok(())
    }
}
