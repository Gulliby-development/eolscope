#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub line: usize,
    pub col: usize,
    pub code: &'static str,
    pub text: String,
}

impl Problem {
    pub fn render(&self, display: &str) -> String {
        format!(
            "{}:{}:{}: {} {}",
            display, self.line, self.col, self.code, self.text
        )
    }
}

pub fn sorted(mut found: Vec<Problem>) -> Vec<Problem> {
    found.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then_with(|| left.col.cmp(&right.col))
            .then_with(|| left.code.cmp(right.code))
    });
    found
}
