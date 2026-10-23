#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Lf,
    Go,
    Rust,
    Universal,
    Js,
    Python,
    Unicode,
}

impl Profile {
    pub const ALL: [Profile; 7] = [
        Profile::Lf,
        Profile::Go,
        Profile::Rust,
        Profile::Universal,
        Profile::Js,
        Profile::Python,
        Profile::Unicode,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Profile::Lf => "lf",
            Profile::Go => "go",
            Profile::Rust => "rust",
            Profile::Universal => "universal",
            Profile::Js => "js",
            Profile::Python => "python",
            Profile::Unicode => "unicode",
        }
    }

    pub fn parse(name: &str) -> Option<Profile> {
        match name {
            "lf" => Some(Profile::Lf),
            "go" => Some(Profile::Go),
            "rust" => Some(Profile::Rust),
            "universal" => Some(Profile::Universal),
            "js" => Some(Profile::Js),
            "python" => Some(Profile::Python),
            "unicode" => Some(Profile::Unicode),
            _ => None,
        }
    }

    pub fn ends_line(self, c: char) -> bool {
        match self {
            Profile::Lf | Profile::Go | Profile::Rust => c == '\n',
            Profile::Universal => c == '\n' || c == '\r',
            Profile::Js => c == '\n' || c == '\r' || c == '\u{2028}' || c == '\u{2029}',
            Profile::Python => {
                c == '\n'
                    || c == '\r'
                    || c == '\u{000b}'
                    || c == '\u{000c}'
                    || c == '\u{001c}'
                    || c == '\u{001d}'
                    || c == '\u{001e}'
                    || c == '\u{0085}'
                    || c == '\u{2028}'
                    || c == '\u{2029}'
            }
            Profile::Unicode => {
                c == '\n'
                    || c == '\r'
                    || c == '\u{000b}'
                    || c == '\u{000c}'
                    || c == '\u{0085}'
                    || c == '\u{2028}'
                    || c == '\u{2029}'
            }
        }
    }
}
