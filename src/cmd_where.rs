use crate::error::EsError;
use crate::io::{Inputs, Io};
use crate::options;
use crate::profile::Profile;
use crate::splitter::{self, Line};
use crate::text::{plural, show, table};

pub const USAGE: &str = "where OFFSET FILE";

pub fn locate(text: &str, profile: Profile, offset: usize) -> (usize, usize, String) {
    let lines = splitter::split(text, profile);
    let text_length = text.chars().count();

    if offset >= text_length {
        if let Some(last_line) = lines.last()
            && last_line.kind.is_none()
        {
            return (
                lines.len(),
                last_line.content.chars().count().saturating_add(1),
                "end".to_string(),
            );
        }
        return (lines.len().saturating_add(1), 1, "end".to_string());
    }

    for (index, line) in lines.iter().enumerate() {
        if let Some(location) = location_in_line(line, profile, offset, index) {
            return location;
        }
    }

    (lines.len().saturating_add(1), 1, "end".to_string())
}

fn location_in_line(
    line: &Line,
    profile: Profile,
    offset: usize,
    index: usize,
) -> Option<(usize, usize, String)> {
    let line_number = index.saturating_add(1);
    let content_length = line.content.chars().count();
    let content_end = line.start.saturating_add(content_length);

    if offset >= line.start && offset < content_end {
        return Some((
            line_number,
            offset.saturating_sub(line.start).saturating_add(1),
            "text".to_string(),
        ));
    }

    if offset >= content_end && offset < line.end {
        let column = content_length.saturating_add(1);
        if profile == Profile::Go && line.kind.is_none() {
            return Some((line_number, column, "CR".to_string()));
        }
        if let Some(kind) = line.kind {
            return Some((line_number, column, kind.name().to_string()));
        }
    }

    None
}

pub fn run(args: &[String], io: &dyn Io) -> Result<i32, EsError> {
    let parsed = options::parse(args, &[])?;
    let [offset_arg, file_arg] = parsed.args.as_slice() else {
        return Err(EsError(format!("usage: {USAGE}")));
    };

    let offset = parse_offset(offset_arg)?;
    let mut inputs = Inputs::new(io);
    let source = inputs.read(file_arg)?;
    let text_length = source.text.chars().count();
    if offset > text_length {
        return Err(EsError(format!(
            "offset {offset} is beyond the end ({})",
            plural(text_length, "character")
        )));
    }

    let rows: Vec<Vec<String>> = Profile::ALL
        .iter()
        .map(|profile| {
            let (line, column, at) = locate(&source.text, *profile, offset);
            vec![
                profile.name().to_string(),
                line.to_string(),
                column.to_string(),
                at,
            ]
        })
        .collect();
    let lines = table(&["PROFILE", "LINE", "COL", "AT"], &rows);
    let mut output = String::new();
    for line in lines {
        output.push_str(&line);
        output.push('\n');
    }
    io.out(&output);
    Ok(0)
}

fn parse_offset(argument: &str) -> Result<usize, EsError> {
    let bytes = argument.as_bytes();
    let canonical = match bytes {
        [b'0'] => true,
        [first @ b'1'..=b'9', rest @ ..] if bytes.len() <= 9 => {
            let _ = first;
            rest.iter().all(u8::is_ascii_digit)
        }
        _ => false,
    };
    if !canonical || bytes.len() > 9 {
        return Err(EsError(format!("invalid offset {}", show(argument))));
    }

    argument
        .parse::<usize>()
        .map_err(|_| EsError(format!("invalid offset {}", show(argument))))
}
