use crate::available_color_names;

use super::{Logger, OutputOptions, TraceLogger};

fn print_blank_line() {
    println!();
}

fn print_example_line(option_root: &str, selector: &str, description: Option<&str>) {
    match description {
        Some(description) => println!("  {option_root} '{selector}' {description}"),
        None => println!("  {option_root} '{selector}'"),
    }
}

fn update_output_options<F>(logger: &Logger, update: F) -> Result<(), String>
where
    F: FnOnce(&mut OutputOptions),
{
    let mut options = logger
        .get_output_options()
        .map_err(|error| error.to_string())?;
    update(&mut options);
    logger
        .set_output_options(options)
        .map_err(|error| error.to_string())
}

impl Logger {
    pub fn make_inline_parser(
        &self,
        local_trace_logger: TraceLogger,
        trace_root: impl AsRef<str>,
    ) -> Result<kcli::InlineParser, kcli::ConfigError> {
        let logger = self.clone();
        let local_namespace = local_trace_logger.namespace().to_string();

        let mut parser = kcli::InlineParser::new("trace")?;
        if !trace_root.as_ref().trim().is_empty() {
            parser.set_root(trace_root.as_ref())?;
        }

        {
            let logger = logger.clone();
            let local_namespace = local_namespace.clone();
            parser.set_root_value_handler_with_help(
                move |_context, value| {
                    logger
                        .enable_channels(value, &local_namespace)
                        .map_err(|error| error.to_string())
                },
                "<channels>",
                "Trace selected channels.",
            )?;
        }

        parser.set_flag_handler(
            "-examples",
            |context| {
                let option_root = format!("--{}", context.root);
                print_blank_line();
                println!("General trace selector pattern:");
                println!(
                    "  {} <namespace>.<channel>[.<subchannel>[.<subchannel>]]",
                    option_root
                );
                print_blank_line();
                println!("Trace selector examples:");
                print_example_line(
                    &option_root,
                    ".abc",
                    Some("Select local 'abc' in current namespace"),
                );
                print_example_line(
                    &option_root,
                    ".abc.xyz",
                    Some("Select local nested channel in current namespace"),
                );
                print_example_line(
                    &option_root,
                    "otherapp.channel",
                    Some("Select explicit namespace channel"),
                );
                print_example_line(
                    &option_root,
                    "*.*",
                    Some("Select all <namespace>.<channel> channels"),
                );
                print_example_line(
                    &option_root,
                    "*.*.*",
                    Some("Select all channels up to 2 levels"),
                );
                print_example_line(
                    &option_root,
                    "*.*.*.*",
                    Some("Select all channels up to 3 levels"),
                );
                print_example_line(
                    &option_root,
                    "alpha.*",
                    Some("Select all top-level channels in alpha"),
                );
                print_example_line(
                    &option_root,
                    "alpha.*.*",
                    Some("Select all channels in alpha (up to 2 levels)"),
                );
                print_example_line(
                    &option_root,
                    "alpha.*.*.*",
                    Some("Select all channels in alpha (up to 3 levels)"),
                );
                print_example_line(
                    &option_root,
                    "*.net",
                    Some("Select 'net' across all namespaces"),
                );
                print_example_line(
                    &option_root,
                    "*.scheduler.tick",
                    Some("Select 'scheduler.tick' across namespaces"),
                );
                print_example_line(
                    &option_root,
                    "*.net.*",
                    Some("Select subchannels under 'net' across namespaces"),
                );
                print_example_line(
                    &option_root,
                    "*.{net,io}",
                    Some("Select 'net' and 'io' across all namespaces"),
                );
                print_example_line(
                    &option_root,
                    "{alpha,beta}.*",
                    Some("Select all top-level channels in alpha and beta"),
                );
                print_example_line(&option_root, "alpha.net", None);
                print_example_line(&option_root, "beta.scheduler.tick", None);
                print_example_line(&option_root, "alpha.net,beta.io", None);
                print_example_line(&option_root, "gamma.physics.*", None);
                print_example_line(&option_root, "gamma.physics.*.*", None);
                print_example_line(&option_root, "alpha.{net,cache}", None);
                print_example_line(&option_root, "beta.{io,scheduler}.packet", None);
                print_example_line(&option_root, "{alpha,beta}.net", None);
                print_blank_line();
                Ok(())
            },
            "Show selector examples.",
        )?;

        {
            let logger = logger.clone();
            parser.set_flag_handler(
                "-namespaces",
                move |_context| {
                    let namespaces = logger.get_namespaces().map_err(|error| error.to_string())?;
                    if namespaces.is_empty() {
                        println!("No trace namespaces defined.");
                        print_blank_line();
                        return Ok(());
                    }
                    print_blank_line();
                    println!("Available trace namespaces:");
                    for trace_namespace in namespaces {
                        println!("  {trace_namespace}");
                    }
                    print_blank_line();
                    Ok(())
                },
                "Show initialized trace namespaces.",
            )?;
        }

        {
            let logger = logger.clone();
            parser.set_flag_handler(
                "-channels",
                move |_context| {
                    let namespaces = logger.get_namespaces().map_err(|error| error.to_string())?;
                    let mut printed_any = false;
                    for trace_namespace in namespaces {
                        let channels = logger
                            .get_channels(&trace_namespace)
                            .map_err(|error| error.to_string())?;
                        for channel in channels {
                            if !printed_any {
                                print_blank_line();
                                println!("Available trace channels:");
                                printed_any = true;
                            }
                            println!("  {}.{}", trace_namespace, channel);
                        }
                    }
                    if !printed_any {
                        println!("No trace channels defined.");
                        print_blank_line();
                        return Ok(());
                    }
                    print_blank_line();
                    Ok(())
                },
                "Show initialized trace channels.",
            )?;
        }

        parser.set_flag_handler(
            "-colors",
            |_context| {
                print_blank_line();
                println!("Available trace colors:");
                for color_name in available_color_names() {
                    println!("  {color_name}");
                }
                print_blank_line();
                Ok(())
            },
            "Show available trace colors.",
        )?;

        {
            let logger = logger.clone();
            parser.set_flag_handler(
                "-files",
                move |_context| {
                    update_output_options(&logger, |options| {
                        options.filenames = true;
                        options.line_numbers = true;
                    })
                },
                "Include source file and line in trace output.",
            )?;
        }

        {
            let logger = logger.clone();
            parser.set_flag_handler(
                "-functions",
                move |_context| {
                    update_output_options(&logger, |options| {
                        options.filenames = true;
                        options.line_numbers = true;
                        options.function_names = true;
                    })
                },
                "Include function names in trace output.",
            )?;
        }

        {
            let logger = logger.clone();
            parser.set_flag_handler(
                "-timestamps",
                move |_context| update_output_options(&logger, |options| options.timestamps = true),
                "Include timestamps in trace output.",
            )?;
        }

        Ok(parser)
    }
}
