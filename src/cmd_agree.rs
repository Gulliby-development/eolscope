use crate::error::EsError;
use crate::io::{Inputs, Io};
use crate::options;
use crate::profile::Profile;
use crate::splitter;

pub const USAGE: &str = "agree FILE...";

pub fn summary(display: &str, text: &str) -> (String, bool) {
    let counts = Profile::ALL.map(|profile| splitter::split(text, profile).len());
    let agree = counts.iter().all(|count| *count == counts[0]);
    let mut line = format!("{display}:");

    for (profile, count) in Profile::ALL.iter().zip(counts.iter()) {
        line.push_str(&format!(" {}={count}", profile.name()));
    }

    line.push_str(if agree { " agree" } else { " differ" });
    (line, agree)
}

pub fn run(args: &[String], io: &dyn Io) -> Result<i32, EsError> {
    let parsed = options::parse(args, &[])?;
    if parsed.args.is_empty() {
        return Err(EsError(format!("usage: {USAGE}")));
    }

    let mut inputs = Inputs::new(io);
    let mut output = String::new();
    let mut all_agree = true;

    for argument in &parsed.args {
        let source = inputs.read(argument)?;
        let (line, agree) = summary(&source.display, &source.text);
        if !agree {
            all_agree = false;
        }
        output.push_str(&line);
        output.push('\n');
    }

    io.out(&output);
    Ok(if all_agree { 0 } else { 1 })
}
