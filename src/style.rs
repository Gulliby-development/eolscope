#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Lf,
    Crlf,
    Cr,
    Nel,
    Ls,
    Ps,
}

impl Style {
    pub const ALL: [Style; 6] = [
        Style::Lf,
        Style::Crlf,
        Style::Cr,
        Style::Nel,
        Style::Ls,
        Style::Ps,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Style::Lf => "lf",
            Style::Crlf => "crlf",
            Style::Cr => "cr",
            Style::Nel => "nel",
            Style::Ls => "ls",
            Style::Ps => "ps",
        }
    }

    pub fn parse(name: &str) -> Option<Style> {
        match name {
            "lf" => Some(Style::Lf),
            "crlf" => Some(Style::Crlf),
            "cr" => Some(Style::Cr),
            "nel" => Some(Style::Nel),
            "ls" => Some(Style::Ls),
            "ps" => Some(Style::Ps),
            _ => None,
        }
    }

    pub fn text(self) -> &'static str {
        match self {
            Style::Lf => "\n",
            Style::Crlf => "\r\n",
            Style::Cr => "\r",
            Style::Nel => "\u{0085}",
            Style::Ls => "\u{2028}",
            Style::Ps => "\u{2029}",
        }
    }

    pub fn kind_name(self) -> &'static str {
        match self {
            Style::Lf => "LF",
            Style::Crlf => "CRLF",
            Style::Cr => "CR",
            Style::Nel => "NEL",
            Style::Ls => "LS",
            Style::Ps => "PS",
        }
    }
}
