use crate::convert::convert;
use crate::error::EsError;
use crate::io::{Inputs, Io};
use crate::options;
use crate::profile::Profile;
use crate::style::Style;
use crate::text::show;

pub const USAGE: &str = "convert [--final-newline] [--strip-bom] FROM TO FILE";

pub fn run(args: &[String], io: &dyn Io) -> Result<i32, EsError> {
    let parsed = options::parse(args, &["--final-newline", "--strip-bom"])?;
    let [from_name, to_name, file] = parsed.args.as_slice() else {
        return Err(EsError(format!("usage: {USAGE}")));
    };

    let from = Profile::parse(from_name)
        .ok_or_else(|| EsError(format!("unknown profile {}", show(from_name))))?;
    let to =
        Style::parse(to_name).ok_or_else(|| EsError(format!("unknown style {}", show(to_name))))?;
    let source = Inputs::new(io).read(file)?;
    let output = convert(
        &source.text,
        from,
        to,
        parsed.flags.contains("--final-newline"),
        parsed.flags.contains("--strip-bom"),
    );

    io.out(&output);
    Ok(0)
}
