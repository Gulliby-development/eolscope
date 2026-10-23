use eolscope::error::EsError;
use eolscope::policy::{Policy, parse};
use eolscope::problem::{Problem, sorted};

#[test]
fn public_types_expose_their_fields_and_derives() {
    let policy = Policy {
        style: Some("lf".to_string()),
        final_newline: Some(true),
        bom: Some(false),
        notrail: true,
    };
    assert_eq!(
        format!("{policy:?}"),
        "Policy { style: Some(\"lf\"), final_newline: Some(true), bom: Some(false), notrail: true }"
    );
    assert_eq!(policy.clone(), policy);
    assert_ne!(
        policy,
        Policy {
            style: Some("cr".to_string()),
            final_newline: Some(true),
            bom: Some(false),
            notrail: true,
        }
    );
    assert_ne!(
        policy,
        Policy {
            style: Some("lf".to_string()),
            final_newline: Some(false),
            bom: Some(false),
            notrail: true,
        }
    );
    assert_ne!(
        policy,
        Policy {
            style: Some("lf".to_string()),
            final_newline: Some(true),
            bom: Some(true),
            notrail: true,
        }
    );
    assert_ne!(
        policy,
        Policy {
            style: Some("lf".to_string()),
            final_newline: Some(true),
            bom: Some(false),
            notrail: false,
        }
    );

    let problem = Problem {
        line: 1,
        col: 2,
        code: "EV001",
        text: "example".to_string(),
    };
    assert_eq!(
        format!("{problem:?}"),
        "Problem { line: 1, col: 2, code: \"EV001\", text: \"example\" }"
    );
    assert_eq!(problem.clone(), problem);
    assert_ne!(
        problem,
        Problem {
            line: 2,
            col: 2,
            code: "EV001",
            text: "example".to_string(),
        }
    );
    assert_ne!(
        problem,
        Problem {
            line: 1,
            col: 3,
            code: "EV001",
            text: "example".to_string(),
        }
    );
    assert_ne!(
        problem,
        Problem {
            line: 1,
            col: 2,
            code: "EV002",
            text: "example".to_string(),
        }
    );
    assert_ne!(
        problem,
        Problem {
            line: 1,
            col: 2,
            code: "EV001",
            text: "different".to_string(),
        }
    );
}

#[test]
fn parser_accepts_each_exact_style() {
    for (input, expected) in [
        (
            "lf",
            Policy {
                style: Some("lf".to_string()),
                final_newline: None,
                bom: None,
                notrail: false,
            },
        ),
        (
            "crlf",
            Policy {
                style: Some("crlf".to_string()),
                final_newline: None,
                bom: None,
                notrail: false,
            },
        ),
        (
            "cr",
            Policy {
                style: Some("cr".to_string()),
                final_newline: None,
                bom: None,
                notrail: false,
            },
        ),
        (
            "nel",
            Policy {
                style: Some("nel".to_string()),
                final_newline: None,
                bom: None,
                notrail: false,
            },
        ),
        (
            "ls",
            Policy {
                style: Some("ls".to_string()),
                final_newline: None,
                bom: None,
                notrail: false,
            },
        ),
        (
            "ps",
            Policy {
                style: Some("ps".to_string()),
                final_newline: None,
                bom: None,
                notrail: false,
            },
        ),
    ] {
        assert_eq!(parse(input), Ok(expected));
    }
}

#[test]
fn parser_sets_optional_flags_and_preserves_defaults() {
    assert_eq!(
        parse("final,bom,notrail"),
        Ok(Policy {
            style: None,
            final_newline: Some(true),
            bom: Some(true),
            notrail: true,
        })
    );
    assert_eq!(
        parse("nofinal,nobom"),
        Ok(Policy {
            style: None,
            final_newline: Some(false),
            bom: Some(false),
            notrail: false,
        })
    );
    assert_eq!(
        parse("lf"),
        Ok(Policy {
            style: Some("lf".to_string()),
            final_newline: None,
            bom: None,
            notrail: false,
        })
    );
}

#[test]
fn parser_makes_identical_repeats_idempotent() {
    assert_eq!(
        parse("lf,final,bom,notrail,lf,final,bom,notrail"),
        Ok(Policy {
            style: Some("lf".to_string()),
            final_newline: Some(true),
            bom: Some(true),
            notrail: true,
        })
    );
    assert_eq!(
        parse("nofinal,nobom,nofinal,nobom"),
        Ok(Policy {
            style: None,
            final_newline: Some(false),
            bom: Some(false),
            notrail: false,
        })
    );
}

#[test]
fn parser_reports_empty_policy_and_comma_items() {
    for (input, expected) in [
        ("", "empty policy"),
        (",lf", "empty item 1"),
        ("lf,", "empty item 2"),
        ("lf,,final", "empty item 2"),
    ] {
        assert_eq!(parse(input), Err(EsError(expected.to_string())));
    }
}

#[test]
fn parser_reports_exact_unknown_items() {
    for (input, expected) in [
        ("lf,CRLF", "unknown item 2 \"CRLF\""),
        ("lf, final", "unknown item 2 \" final\""),
        ("lf ", "unknown item 1 \"lf \""),
    ] {
        assert_eq!(parse(input), Err(EsError(expected.to_string())));
    }
}

#[test]
fn parser_shows_unrecognized_control_unicode_and_quote_items() {
    for (input, expected) in [
        ("lf,\t", "unknown item 2 \"?\""),
        ("lf,😀", "unknown item 2 \"?\""),
        ("lf,a\"b", "unknown item 2 \"a\"b\""),
    ] {
        assert_eq!(parse(input), Err(EsError(expected.to_string())));
    }
}

#[test]
fn parser_reports_first_style_and_final_conflicts() {
    for (input, expected) in [
        ("lf,crlf", "two line styles lf and crlf"),
        ("ps,lf", "two line styles ps and lf"),
        ("final,nofinal,x", "conflicting items final and nofinal"),
        ("nofinal,final", "conflicting items nofinal and final"),
        ("bom,nobom", "conflicting items bom and nobom"),
    ] {
        assert_eq!(parse(input), Err(EsError(expected.to_string())));
    }
}

#[test]
fn parser_preserves_first_error_and_bom_conflict_order() {
    for (input, expected) in [
        ("nobom,bom", "conflicting items nobom and bom"),
        ("lf,final,lf,cr", "two line styles lf and cr"),
        ("x,final,nofinal", "unknown item 1 \"x\""),
        ("lf,,crlf", "empty item 2"),
        ("lf,crlf,", "two line styles lf and crlf"),
    ] {
        assert_eq!(parse(input), Err(EsError(expected.to_string())));
    }
}

#[test]
fn problem_render_keeps_all_supplied_values_verbatim() {
    for (problem, display, expected) in [
        (
            Problem {
                line: 2,
                col: 3,
                code: "EV001",
                text: "CRLF terminator, expected LF".to_string(),
            },
            "t.txt",
            "t.txt:2:3: EV001 CRLF terminator, expected LF",
        ),
        (
            Problem {
                line: 1,
                col: 1,
                code: "EV004",
                text: "byte order mark found".to_string(),
            },
            "<stdin>",
            "<stdin>:1:1: EV004 byte order mark found",
        ),
        (
            Problem {
                line: 0,
                col: 0,
                code: "X",
                text: "".to_string(),
            },
            "café",
            "café:0:0: X ",
        ),
    ] {
        assert_eq!(problem.render(display), expected);
    }
}

#[test]
fn sorted_keeps_an_empty_vector_empty() {
    assert_eq!(sorted(Vec::new()), Vec::<Problem>::new());
}

#[test]
fn sorted_preserves_a_single_problem() {
    let problem = Problem {
        line: 1,
        col: 1,
        code: "EV001",
        text: "only".to_string(),
    };
    assert_eq!(sorted(vec![problem.clone()]), vec![problem]);
}

#[test]
fn sorted_orders_line_column_code_and_keeps_equal_key_order() {
    assert_eq!(
        sorted(vec![
            Problem {
                line: 2,
                col: 1,
                code: "EV001",
                text: "later line".to_string(),
            },
            Problem {
                line: 1,
                col: 2,
                code: "EV001",
                text: "later column".to_string(),
            },
            Problem {
                line: 1,
                col: 1,
                code: "EV006",
                text: "later code".to_string(),
            },
            Problem {
                line: 1,
                col: 1,
                code: "EV001",
                text: "z first tie".to_string(),
            },
            Problem {
                line: 1,
                col: 1,
                code: "EV001",
                text: "a second tie".to_string(),
            },
        ]),
        vec![
            Problem {
                line: 1,
                col: 1,
                code: "EV001",
                text: "z first tie".to_string(),
            },
            Problem {
                line: 1,
                col: 1,
                code: "EV001",
                text: "a second tie".to_string(),
            },
            Problem {
                line: 1,
                col: 1,
                code: "EV006",
                text: "later code".to_string(),
            },
            Problem {
                line: 1,
                col: 2,
                code: "EV001",
                text: "later column".to_string(),
            },
            Problem {
                line: 2,
                col: 1,
                code: "EV001",
                text: "later line".to_string(),
            },
        ]
    );
}
