use crate::error::EsError;
use crate::io::{Inputs, Io};
use crate::kind::Kind;
use crate::options;
use crate::profile::Profile;
use crate::splitter;
use crate::text::{self, show};

pub const USAGE: &str = "lines PROFILE FILE";

pub fn render(text: &str, profile: Profile) -> Vec<String> {
    let lines = splitter::split(text, profile);
    if lines.is_empty() {
        return vec!["none".to_string()];
    }

    let rows = lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            vec![
                index.saturating_add(1).to_string(),
                line.kind.map_or("none", Kind::name).to_string(),
                line.content.chars().count().to_string(),
                show(&line.content),
            ]
        })
        .collect::<Vec<_>>();

    text::table(&["LINE", "END", "CHARS", "TEXT"], &rows)
}

pub fn run(args: &[String], io: &dyn Io) -> Result<i32, EsError> {
    let parsed = options::parse(args, &[])?;
    let [profile_name, file_name] = parsed.args.as_slice() else {
        return Err(EsError(format!("usage: {USAGE}")));
    };

    let profile = Profile::parse(profile_name)
        .ok_or_else(|| EsError(format!("unknown profile {}", text::show(profile_name))))?;

    let source = Inputs::new(io).read(file_name)?;
    let output = render(&source.text, profile).join("\n");
    io.out(&format!("{output}\n"));
    Ok(0)
}
