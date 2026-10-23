#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    JsTrim,
    PyStrip,
    RsTrim,
    JavaTrim,
    JavaStrip,
}

impl Rule {
    pub const ALL: [Rule; 5] = [
        Rule::JsTrim,
        Rule::PyStrip,
        Rule::RsTrim,
        Rule::JavaTrim,
        Rule::JavaStrip,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Rule::JsTrim => "js-trim",
            Rule::PyStrip => "py-strip",
            Rule::RsTrim => "rs-trim",
            Rule::JavaTrim => "java-trim",
            Rule::JavaStrip => "java-strip",
        }
    }

    pub fn strips(self, c: char) -> bool {
        match self {
            Rule::JsTrim => matches!(
                c,
                '\u{0009}'..='\u{000D}'
                    | '\u{0020}'
                    | '\u{00A0}'
                    | '\u{1680}'
                    | '\u{2000}'..='\u{200A}'
                    | '\u{2028}'
                    | '\u{2029}'
                    | '\u{202F}'
                    | '\u{205F}'
                    | '\u{3000}'
                    | '\u{FEFF}'
            ),
            Rule::PyStrip => matches!(
                c,
                '\u{0009}'..='\u{000D}'
                    | '\u{001C}'..='\u{001F}'
                    | '\u{0020}'
                    | '\u{0085}'
                    | '\u{00A0}'
                    | '\u{1680}'
                    | '\u{2000}'..='\u{200A}'
                    | '\u{2028}'
                    | '\u{2029}'
                    | '\u{202F}'
                    | '\u{205F}'
                    | '\u{3000}'
            ),
            Rule::RsTrim => matches!(
                c,
                '\u{0009}'..='\u{000D}'
                    | '\u{0020}'
                    | '\u{0085}'
                    | '\u{00A0}'
                    | '\u{1680}'
                    | '\u{2000}'..='\u{200A}'
                    | '\u{2028}'
                    | '\u{2029}'
                    | '\u{202F}'
                    | '\u{205F}'
                    | '\u{3000}'
            ),
            Rule::JavaTrim => matches!(c, '\u{0000}'..='\u{0020}'),
            Rule::JavaStrip => matches!(
                c,
                '\u{0009}'..='\u{000D}'
                    | '\u{001C}'..='\u{001F}'
                    | '\u{0020}'
                    | '\u{1680}'
                    | '\u{2000}'..='\u{2006}'
                    | '\u{2008}'..='\u{200A}'
                    | '\u{2028}'
                    | '\u{2029}'
                    | '\u{205F}'
                    | '\u{3000}'
            ),
        }
    }
}
