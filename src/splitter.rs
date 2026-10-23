use crate::kind::Kind;
use crate::profile::Profile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub content: String,
    pub kind: Option<Kind>,
    pub start: usize,
    pub end: usize,
}

pub fn split(text: &str, profile: Profile) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut chars = text.chars().peekable();
    let mut content = String::new();
    let mut line_start = 0;
    let mut offset = 0;
    let mut previous_was_cr = false;

    while let Some(c) = chars.next() {
        if profile.ends_line(c) {
            if c == '\r' && profile.ends_line('\r') && chars.peek() == Some(&'\n') {
                let _ = chars.next();
                offset += 2;
                lines.push(Line {
                    content: std::mem::take(&mut content),
                    kind: Some(Kind::Crlf),
                    start: line_start,
                    end: offset,
                });
                line_start = offset;
                previous_was_cr = false;
                continue;
            }

            if c == '\n' && matches!(profile, Profile::Go | Profile::Rust) && previous_was_cr {
                let _ = content.pop();
                offset += 1;
                lines.push(Line {
                    content: std::mem::take(&mut content),
                    kind: Some(Kind::Crlf),
                    start: line_start,
                    end: offset,
                });
                line_start = offset;
                previous_was_cr = false;
                continue;
            }

            offset += 1;
            lines.push(Line {
                content: std::mem::take(&mut content),
                kind: Kind::of_char(c),
                start: line_start,
                end: offset,
            });
            line_start = offset;
            previous_was_cr = false;
        } else {
            content.push(c);
            offset += 1;
            previous_was_cr = c == '\r';
        }
    }

    if offset > line_start {
        if matches!(profile, Profile::Go) && content.ends_with('\r') {
            let _ = content.pop();
        }
        lines.push(Line {
            content,
            kind: None,
            start: line_start,
            end: offset,
        });
    }

    lines
}
