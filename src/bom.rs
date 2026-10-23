pub fn has_bom(text: &str) -> bool {
    text.starts_with('\u{FEFF}')
}

pub fn strip_bom(text: &str) -> &str {
    match text.strip_prefix('\u{FEFF}') {
        Some(stripped) => stripped,
        None => text,
    }
}
