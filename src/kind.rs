#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Lf,
    Crlf,
    Cr,
    Vt,
    Ff,
    Fs,
    Gs,
    Rs,
    Nel,
    Ls,
    Ps,
}

impl Kind {
    pub const ALL: [Kind; 11] = [
        Kind::Lf,
        Kind::Crlf,
        Kind::Cr,
        Kind::Vt,
        Kind::Ff,
        Kind::Fs,
        Kind::Gs,
        Kind::Rs,
        Kind::Nel,
        Kind::Ls,
        Kind::Ps,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Kind::Lf => "LF",
            Kind::Crlf => "CRLF",
            Kind::Cr => "CR",
            Kind::Vt => "VT",
            Kind::Ff => "FF",
            Kind::Fs => "FS",
            Kind::Gs => "GS",
            Kind::Rs => "RS",
            Kind::Nel => "NEL",
            Kind::Ls => "LS",
            Kind::Ps => "PS",
        }
    }

    pub fn of_char(c: char) -> Option<Kind> {
        match c {
            '\n' => Some(Kind::Lf),
            '\r' => Some(Kind::Cr),
            '\u{000b}' => Some(Kind::Vt),
            '\u{000c}' => Some(Kind::Ff),
            '\u{001c}' => Some(Kind::Fs),
            '\u{001d}' => Some(Kind::Gs),
            '\u{001e}' => Some(Kind::Rs),
            '\u{0085}' => Some(Kind::Nel),
            '\u{2028}' => Some(Kind::Ls),
            '\u{2029}' => Some(Kind::Ps),
            _ => None,
        }
    }
}
