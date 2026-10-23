use crate::bom::has_bom;
use crate::error::EsError;
use crate::io::{Inputs, Io};
use crate::kind::Kind;
use crate::options;
use crate::profile::Profile;
use crate::splitter::split;

pub const USAGE: &str = "kinds FILE...";

pub fn report(display: &str, text: &str) -> (Vec<String>, bool) {
    let lines = split(text, Profile::Python);
    let mut occurring = Vec::new();

    for kind in Kind::ALL {
        let count = lines.iter().filter(|line| line.kind == Some(kind)).count();
        if count > 0 {
            occurring.push((kind, count));
        }
    }

    let mixed = occurring.len() > 1;
    let style = match occurring.as_slice() {
        [] => "none".to_string(),
        [(kind, _)] => kind.name().to_string(),
        _ => "mixed".to_string(),
    };

    let final_state = if text.is_empty() {
        "-".to_string()
    } else if lines.last().is_some_and(|line| line.kind.is_some()) {
        "yes".to_string()
    } else {
        "no".to_string()
    };

    let mut rows = Vec::new();
    rows.push(("lines".to_string(), lines.len().to_string()));
    for (kind, count) in occurring {
        rows.push((kind.name().to_string(), count.to_string()));
    }
    rows.push(("style".to_string(), style));
    rows.push(("final".to_string(), final_state));
    rows.push((
        "bom".to_string(),
        if has_bom(text) { "yes" } else { "no" }.to_string(),
    ));

    let label_width = rows
        .iter()
        .map(|(label, _)| label.len())
        .fold(0, usize::max);
    let mut output = Vec::with_capacity(rows.len() + 1);
    output.push(format!("{display}:"));
    for (label, value) in rows {
        output.push(format!("  {label:<label_width$}  {value}"));
    }

    (output, mixed)
}

pub fn run(args: &[String], io: &dyn Io) -> Result<i32, EsError> {
    let parsed = options::parse(args, &[])?;
    if parsed.args.is_empty() {
        return Err(EsError(format!("usage: {USAGE}")));
    }

    let mut inputs = Inputs::new(io);
    let mut output = String::new();
    let mut mixed = false;

    for arg in &parsed.args {
        let source = inputs.read(arg)?;
        let (report_lines, file_mixed) = report(&source.display, &source.text);
        mixed |= file_mixed;
        for line in report_lines {
            output.push_str(&line);
            output.push('\n');
        }
    }

    io.out(&output);
    Ok(if mixed { 1 } else { 0 })
}
