use crate::bom::strip_bom as remove_bom;
use crate::profile::Profile;
use crate::splitter::split;
use crate::style::Style;

pub fn convert(
    text: &str,
    from: Profile,
    to: Style,
    final_newline: bool,
    strip_bom: bool,
) -> String {
    let text = if strip_bom { remove_bom(text) } else { text };
    let mut converted = String::new();

    for line in split(text, from) {
        converted.push_str(&line.content);
        if line.kind.is_some() || final_newline {
            converted.push_str(to.text());
        }
    }

    converted
}
