use crate::error::EsError;
use crate::io::Io;
use crate::options;
use crate::profile::Profile;
use crate::rule::Rule;
use crate::text::{self, show};

pub const USAGE: &str = "chars [CODE...]";

pub const SPECIALS: [(u32, &str); 19] = [
    (0x0009, "TAB"),
    (0x000A, "LF"),
    (0x000B, "VT"),
    (0x000C, "FF"),
    (0x000D, "CR"),
    (0x001C, "FS"),
    (0x001D, "GS"),
    (0x001E, "RS"),
    (0x001F, "US"),
    (0x0020, "SPACE"),
    (0x0085, "NEL"),
    (0x00A0, "NBSP"),
    (0x2007, "FIGURESP"),
    (0x2028, "LS"),
    (0x2029, "PS"),
    (0x202F, "NNBSP"),
    (0x3000, "IDEOSP"),
    (0xFEFF, "BOM"),
    (0x200B, "ZWSP"),
];

pub fn row(code: u32) -> Vec<String> {
    let name = SPECIALS
        .iter()
        .find(|(special, _)| *special == code)
        .map(|(_, name)| *name)
        .unwrap_or("-");
    let character = char::from_u32(code);

    let mut cells = vec![format!("U+{code:04X}"), name.to_string()];
    cells.extend(Profile::ALL.iter().map(|profile| {
        if character
            .map(|value| profile.ends_line(value))
            .unwrap_or(false)
        {
            "T".to_string()
        } else {
            ".".to_string()
        }
    }));
    cells.extend(Rule::ALL.iter().map(|rule| {
        if character.map(|value| rule.strips(value)).unwrap_or(false) {
            "x".to_string()
        } else {
            ".".to_string()
        }
    }));
    cells
}

pub fn run(args: &[String], io: &dyn Io) -> Result<i32, EsError> {
    let parsed = options::parse(args, &[])?;
    let codes = if parsed.args.is_empty() {
        SPECIALS.iter().map(|(code, _)| *code).collect()
    } else {
        let mut codes = Vec::with_capacity(parsed.args.len());
        for argument in &parsed.args {
            match parse_code(argument) {
                Some(code) => codes.push(code),
                None => return Err(EsError(format!("invalid code {}", show(argument)))),
            }
        }
        codes
    };

    let mut header = vec!["CODE", "NAME"];
    header.extend(Profile::ALL.iter().map(|profile| profile.name()));
    header.extend(Rule::ALL.iter().map(|rule| rule.name()));

    let rows: Vec<Vec<String>> = codes.into_iter().map(row).collect();
    let mut output = text::table(&header, &rows).join("\n");
    output.push('\n');
    io.out(&output);
    Ok(0)
}

fn parse_code(argument: &str) -> Option<u32> {
    let digits = match argument
        .strip_prefix("U+")
        .or_else(|| argument.strip_prefix("u+"))
    {
        Some(digits) => digits,
        None => argument,
    };

    if !(4..=6).contains(&digits.len()) || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }

    let code = u32::from_str_radix(digits, 16).ok()?;
    char::from_u32(code)?;
    Some(code)
}
