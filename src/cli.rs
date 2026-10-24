use crate::error::EsError;
use crate::io::Io;
use crate::text::show;

pub const USAGE: &str = "usage: eolscope lines PROFILE FILE\nusage: eolscope agree FILE...\nusage: eolscope kinds FILE...\nusage: eolscope where OFFSET FILE\nusage: eolscope trim [--all] PROFILE FILE...\nusage: eolscope chars [CODE...]\nusage: eolscope convert [--final-newline] [--strip-bom] FROM TO FILE\nusage: eolscope check POLICY FILE...\n";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn run(argv: &[String], io: &dyn Io) -> i32 {
    if matches!(argv, [argument] if argument == "--help") {
        io.out(USAGE);
        return 0;
    }

    if matches!(argv, [argument] if argument == "--version") {
        io.out(&format!("eolscope {VERSION}\n"));
        return 0;
    }

    let result = match argv.split_first() {
        None => Err(EsError(
            "missing command (lines, agree, kinds, where, trim, chars, convert or check)"
                .to_string(),
        )),
        Some((command, args)) => match command.as_str() {
            "lines" => crate::cmd_lines::run(args, io),
            "agree" => crate::cmd_agree::run(args, io),
            "kinds" => crate::cmd_kinds::run(args, io),
            "where" => crate::cmd_where::run(args, io),
            "trim" => crate::cmd_trim::run(args, io),
            "chars" => crate::cmd_chars::run(args, io),
            "convert" => crate::cmd_convert::run(args, io),
            "check" => crate::cmd_check::run(args, io),
            _ => Err(EsError(format!("unknown command {}", show(command)))),
        },
    };

    match result {
        Ok(status) => status,
        Err(EsError(message)) => {
            io.err(&format!("eolscope: {message}\n"));
            2
        }
    }
}
