use crate::error::EsError;
use crate::io::{Inputs, Io};
use crate::options;
use crate::profile::Profile;
use crate::rule::Rule;
use crate::splitter;
use crate::tail::trailing;
use crate::text::{self, show};

pub const USAGE: &str = "trim [--all] PROFILE FILE...";

const HEADER: [&str; 8] = [
    "FILE",
    "LINE",
    "js-trim",
    "py-strip",
    "rs-trim",
    "java-trim",
    "java-strip",
    "VERDICT",
];

pub fn run(args: &[String], io: &dyn Io) -> Result<i32, EsError> {
    let parsed = options::parse(args, &["--all"])?;
    if parsed.args.len() < 2 {
        return Err(EsError(format!("usage: {USAGE}")));
    }

    let profile_name = parsed
        .args
        .first()
        .ok_or_else(|| EsError(format!("usage: {USAGE}")))?;
    let profile = Profile::parse(profile_name)
        .ok_or_else(|| EsError(format!("unknown profile {}", show(profile_name))))?;
    let include_all = parsed.flags.contains("--all");

    let mut inputs = Inputs::new(io);
    let mut rows = Vec::new();
    let mut differs = false;

    for path in parsed.args.iter().skip(1) {
        let source = inputs.read(path)?;
        for (index, line) in splitter::split(&source.text, profile).iter().enumerate() {
            let counts = Rule::ALL
                .into_iter()
                .map(|rule| trailing(rule, &line.content))
                .collect::<Vec<_>>();
            let first_count = counts.first().copied().unwrap_or(0);
            let same = counts.iter().all(|&count| count == first_count);

            if !same {
                differs = true;
            }
            if !same || include_all {
                let mut row = Vec::with_capacity(HEADER.len());
                row.push(source.display.clone());
                row.push((index + 1).to_string());
                row.extend(counts.into_iter().map(|count| count.to_string()));
                row.push(if same { "same" } else { "differ" }.to_string());
                rows.push(row);
            }
        }
    }

    let mut output = String::new();
    if rows.is_empty() {
        output.push_str("none\n");
    } else {
        for line in text::table(&HEADER, &rows) {
            output.push_str(&line);
            output.push('\n');
        }
    }

    io.out(&output);
    Ok(if differs { 1 } else { 0 })
}
