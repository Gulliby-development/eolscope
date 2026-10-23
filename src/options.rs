use std::collections::BTreeSet;

use crate::error::EsError;
use crate::text::show;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed {
    pub flags: BTreeSet<String>,
    pub args: Vec<String>,
}

pub fn parse(argv: &[String], specs: &[&str]) -> Result<Parsed, EsError> {
    let mut flags = BTreeSet::new();
    let mut args = Vec::new();

    for arg in argv {
        if arg.starts_with("--") {
            if specs.contains(&arg.as_str()) {
                flags.insert(arg.clone());
            } else {
                return Err(EsError(format!("unknown option {}", show(arg))));
            }
        } else {
            args.push(arg.clone());
        }
    }

    Ok(Parsed { flags, args })
}
