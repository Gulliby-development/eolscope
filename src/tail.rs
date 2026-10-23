use crate::rule::Rule;

pub fn trailing(rule: Rule, text: &str) -> usize {
    text.chars().rev().take_while(|&c| rule.strips(c)).count()
}
