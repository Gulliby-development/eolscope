use crate::bom::has_bom;
use crate::policy::Policy;
use crate::problem::{Problem, sorted};
use crate::profile::Profile;
use crate::rule::Rule;
use crate::splitter::split;
use crate::style::Style;
use crate::tail::trailing;
use crate::text::plural;

pub fn analyze(text: &str, policy: &Policy) -> Vec<Problem> {
    if text.is_empty() {
        return Vec::new();
    }

    let lines = split(text, Profile::Python);
    let expected_style = policy
        .style
        .as_deref()
        .and_then(Style::parse)
        .map(Style::kind_name);
    let mut problems = Vec::new();

    if let Some(expected) = expected_style {
        for (index, line) in lines.iter().enumerate() {
            if let Some(kind) = line.kind {
                let actual = kind.name();
                if actual != expected {
                    problems.push(Problem {
                        line: index.saturating_add(1),
                        col: line.content.chars().count().saturating_add(1),
                        code: "EV001",
                        text: format!("{actual} terminator, expected {expected}"),
                    });
                }
            }
        }
    }

    if let Some(require_final) = policy.final_newline
        && let Some(last) = lines.last()
    {
        if require_final && last.kind.is_none() {
            problems.push(Problem {
                line: lines.len(),
                col: last.content.chars().count().saturating_add(1),
                code: "EV002",
                text: "missing final line terminator".to_string(),
            });
        } else if !require_final && last.kind.is_some() {
            problems.push(Problem {
                line: lines.len(),
                col: last.content.chars().count().saturating_add(1),
                code: "EV003",
                text: "unexpected final line terminator".to_string(),
            });
        }
    }

    if let Some(require_bom) = policy.bom {
        let found_bom = has_bom(text);
        if !require_bom && found_bom {
            problems.push(Problem {
                line: 1,
                col: 1,
                code: "EV004",
                text: "byte order mark found".to_string(),
            });
        } else if require_bom && !found_bom {
            problems.push(Problem {
                line: 1,
                col: 1,
                code: "EV005",
                text: "byte order mark missing".to_string(),
            });
        }
    }

    if policy.notrail {
        for (index, line) in lines.iter().enumerate() {
            let count = trailing(Rule::RsTrim, &line.content);
            if count > 0 {
                let length = line.content.chars().count();
                problems.push(Problem {
                    line: index.saturating_add(1),
                    col: length.saturating_sub(count).saturating_add(1),
                    code: "EV006",
                    text: format!("trailing whitespace of {}", plural(count, "character")),
                });
            }
        }
    }

    sorted(problems)
}
