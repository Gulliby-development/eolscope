use crate::error::EsError;
use crate::text::show;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    pub style: Option<String>,
    pub final_newline: Option<bool>,
    pub bom: Option<bool>,
    pub notrail: bool,
}

pub fn parse(text: &str) -> Result<Policy, EsError> {
    if text.is_empty() {
        return Err(EsError("empty policy".to_string()));
    }

    let mut policy = Policy {
        style: None,
        final_newline: None,
        bom: None,
        notrail: false,
    };

    for (index, item) in text.split(',').enumerate() {
        let item_number = index.saturating_add(1);
        if item.is_empty() {
            return Err(EsError(format!("empty item {item_number}")));
        }

        match item {
            "lf" | "crlf" | "cr" | "nel" | "ls" | "ps" => {
                if let Some(previous) = &policy.style {
                    if previous != item {
                        return Err(EsError(format!("two line styles {previous} and {item}")));
                    }
                } else {
                    policy.style = Some(item.to_string());
                }
            }
            "final" | "nofinal" => {
                let value = item == "final";
                if let Some(previous) = policy.final_newline {
                    if previous != value {
                        let earlier = if previous { "final" } else { "nofinal" };
                        return Err(EsError(format!("conflicting items {earlier} and {item}")));
                    }
                } else {
                    policy.final_newline = Some(value);
                }
            }
            "bom" | "nobom" => {
                let value = item == "bom";
                if let Some(previous) = policy.bom {
                    if previous != value {
                        let earlier = if previous { "bom" } else { "nobom" };
                        return Err(EsError(format!("conflicting items {earlier} and {item}")));
                    }
                } else {
                    policy.bom = Some(value);
                }
            }
            "notrail" => policy.notrail = true,
            _ => {
                return Err(EsError(format!(
                    "unknown item {item_number} {}",
                    show(item)
                )));
            }
        }
    }

    Ok(policy)
}
