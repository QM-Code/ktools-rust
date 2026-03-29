mod help;
mod plan;
mod state;

use crate::model::{CliError, ParserData};

use self::plan::{execute_invocations, plan_invocations, report_unconsumed_option_tokens};
use self::state::ParseOutcome;

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
        let failure = result
            .error
            .expect("parse failure must carry error details");
        Err(CliError::new(failure.option(), failure.render_message()))
    }
}
