use crate::audit::analyze;
use crate::error::EsError;
use crate::io::{Inputs, Io};
use crate::options::parse;
use crate::policy;
use crate::text::plural;

pub const USAGE: &str = "check POLICY FILE...";

pub fn run(args: &[String], io: &dyn Io) -> Result<i32, EsError> {
    let parsed = parse(args, &[])?;
    if parsed.args.len() < 2 {
        return Err(EsError(format!("usage: {USAGE}")));
    }

    let Some(policy_text) = parsed.args.first() else {
        return Err(EsError(format!("usage: {USAGE}")));
    };
    let checked_policy = policy::parse(policy_text)?;
    let Some(files) = parsed.args.get(1..) else {
        return Err(EsError(format!("usage: {USAGE}")));
    };

    let mut inputs = Inputs::new(io);
    let mut output = String::new();
    let mut total_problems = 0usize;

    for path in files {
        let source = inputs.read(path)?;
        let problems = analyze(&source.text, &checked_policy);
        total_problems = total_problems
            .checked_add(problems.len())
            .ok_or_else(|| EsError("problem count overflow".to_string()))?;

        for problem in problems {
            output.push_str(&problem.render(&source.display));
            output.push('\n');
        }
    }

    if total_problems == 0 {
        output.push_str("ok\n");
    } else {
        output.push_str(&plural(total_problems, "problem"));
        output.push('\n');
    }

    io.out(&output);
    Ok(if total_problems == 0 { 0 } else { 1 })
}
