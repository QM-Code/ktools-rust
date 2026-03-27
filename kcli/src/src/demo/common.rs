use crate::HandlerContext;

pub(crate) fn print_processing_line(context: &HandlerContext, value: &str) {
    if context.value_tokens.is_empty() {
        println!("Processing {}", context.option);
        return;
    }

    if context.value_tokens.len() == 1 {
        println!("Processing {} with value \"{}\"", context.option, value);
        return;
    }

    let joined = context
        .value_tokens
        .iter()
        .map(|token| format!("\"{token}\""))
        .collect::<Vec<_>>()
        .join(",");
    println!("Processing {} with values [{}]", context.option, joined);
}

