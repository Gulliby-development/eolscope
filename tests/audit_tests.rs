use eolscope::audit::analyze;
use eolscope::policy::Policy;
use eolscope::problem::Problem;

#[test]
fn empty_text_is_exempt_under_every_policy_and_clean_text_has_no_problems() {
    assert_eq!(
        analyze(
            "",
            &Policy {
                style: Some("lf".to_string()),
                final_newline: Some(true),
                bom: Some(true),
                notrail: true,
            }
        ),
        Vec::<Problem>::new()
    );
    assert_eq!(
        analyze(
            "",
            &Policy {
                style: Some("crlf".to_string()),
                final_newline: Some(false),
                bom: Some(false),
                notrail: true,
            }
        ),
        Vec::<Problem>::new()
    );
    assert_eq!(
        analyze(
            "a\n",
            &Policy {
                style: Some("lf".to_string()),
                final_newline: Some(true),
                bom: Some(false),
                notrail: true,
            }
        ),
        Vec::<Problem>::new()
    );
}

#[test]
fn bom_only_text_is_not_exempt_and_reports_bom_before_missing_final() {
    assert_eq!(
        analyze(
            "\u{feff}",
            &Policy {
                style: None,
                final_newline: Some(true),
                bom: Some(false),
                notrail: true,
            }
        ),
        vec![
            Problem {
                line: 1,
                col: 1,
                code: "EV004",
                text: "byte order mark found".to_string(),
            },
            Problem {
                line: 1,
                col: 2,
                code: "EV002",
                text: "missing final line terminator".to_string(),
            },
        ]
    );
}

#[test]
fn wrong_lf_style_reports_crlf_and_ls_at_scalar_columns() {
    assert_eq!(
        analyze(
            "a\nb\r\nc\u{2028}d\n",
            &Policy {
                style: Some("lf".to_string()),
                final_newline: None,
                bom: None,
                notrail: false,
            }
        ),
        vec![
            Problem {
                line: 2,
                col: 2,
                code: "EV001",
                text: "CRLF terminator, expected LF".to_string(),
            },
            Problem {
                line: 3,
                col: 2,
                code: "EV001",
                text: "LS terminator, expected LF".to_string(),
            },
        ]
    );
}

#[test]
fn python_only_fs_gs_and_rs_use_astral_scalar_columns() {
    assert_eq!(
        analyze(
            "😀\u{001c}b\u{001d}c\u{001e}",
            &Policy {
                style: Some("lf".to_string()),
                final_newline: None,
                bom: None,
                notrail: false,
            }
        ),
        vec![
            Problem {
                line: 1,
                col: 2,
                code: "EV001",
                text: "FS terminator, expected LF".to_string(),
            },
            Problem {
                line: 2,
                col: 2,
                code: "EV001",
                text: "GS terminator, expected LF".to_string(),
            },
            Problem {
                line: 3,
                col: 2,
                code: "EV001",
                text: "RS terminator, expected LF".to_string(),
            },
        ]
    );
}

#[test]
fn ls_ps_and_nel_are_checked_as_python_line_terminators() {
    assert_eq!(
        analyze(
            "a\u{2028}b\u{2029}c\u{0085}",
            &Policy {
                style: Some("ls".to_string()),
                final_newline: Some(true),
                bom: None,
                notrail: false,
            }
        ),
        vec![
            Problem {
                line: 2,
                col: 2,
                code: "EV001",
                text: "PS terminator, expected LS".to_string(),
            },
            Problem {
                line: 3,
                col: 2,
                code: "EV001",
                text: "NEL terminator, expected LS".to_string(),
            },
        ]
    );
}

#[test]
fn bom_astral_vt_ff_and_cr_report_python_line_positions() {
    assert_eq!(
        analyze(
            "\u{feff}😀\u{000b}\u{000c}\r",
            &Policy {
                style: Some("ps".to_string()),
                final_newline: None,
                bom: None,
                notrail: false,
            }
        ),
        vec![
            Problem {
                line: 1,
                col: 3,
                code: "EV001",
                text: "VT terminator, expected PS".to_string(),
            },
            Problem {
                line: 2,
                col: 1,
                code: "EV001",
                text: "FF terminator, expected PS".to_string(),
            },
            Problem {
                line: 3,
                col: 1,
                code: "EV001",
                text: "CR terminator, expected PS".to_string(),
            },
        ]
    );
}

#[test]
fn final_policy_uses_last_real_line_without_a_phantom_line() {
    assert_eq!(
        analyze(
            "a\nb",
            &Policy {
                style: Some("lf".to_string()),
                final_newline: Some(true),
                bom: None,
                notrail: false,
            }
        ),
        vec![Problem {
            line: 2,
            col: 2,
            code: "EV002",
            text: "missing final line terminator".to_string(),
        }]
    );
    assert_eq!(
        analyze(
            "a\r\nb\r\n",
            &Policy {
                style: Some("crlf".to_string()),
                final_newline: Some(false),
                bom: None,
                notrail: false,
            }
        ),
        vec![Problem {
            line: 2,
            col: 2,
            code: "EV003",
            text: "unexpected final line terminator".to_string(),
        }]
    );
    assert_eq!(
        analyze(
            "a",
            &Policy {
                style: None,
                final_newline: Some(false),
                bom: None,
                notrail: false,
            }
        ),
        Vec::<Problem>::new()
    );
}

#[test]
fn bom_policy_distinguishes_leading_from_embedded_bom() {
    assert_eq!(
        analyze(
            "\u{feff}a\n",
            &Policy {
                style: None,
                final_newline: Some(true),
                bom: Some(false),
                notrail: false,
            }
        ),
        vec![Problem {
            line: 1,
            col: 1,
            code: "EV004",
            text: "byte order mark found".to_string(),
        }]
    );
    assert_eq!(
        analyze(
            "a\u{feff}",
            &Policy {
                style: None,
                final_newline: None,
                bom: Some(true),
                notrail: false,
            }
        ),
        vec![Problem {
            line: 1,
            col: 1,
            code: "EV005",
            text: "byte order mark missing".to_string(),
        }]
    );
    assert_eq!(
        analyze(
            "\u{feff}a",
            &Policy {
                style: None,
                final_newline: None,
                bom: Some(true),
                notrail: false,
            }
        ),
        Vec::<Problem>::new()
    );
}

#[test]
fn notrail_uses_rs_trim_and_points_to_the_first_trailing_scalar() {
    assert_eq!(
        analyze(
            "a \nb\u{00a0}\t\u{2028}c\u{feff}\nd\u{0085}e\u{3000}",
            &Policy {
                style: None,
                final_newline: None,
                bom: None,
                notrail: true,
            }
        ),
        vec![
            Problem {
                line: 1,
                col: 2,
                code: "EV006",
                text: "trailing whitespace of 1 character".to_string(),
            },
            Problem {
                line: 2,
                col: 2,
                code: "EV006",
                text: "trailing whitespace of 2 characters".to_string(),
            },
            Problem {
                line: 5,
                col: 2,
                code: "EV006",
                text: "trailing whitespace of 1 character".to_string(),
            },
        ]
    );
}

#[test]
fn notrail_combines_plural_count_with_bom_and_final_diagnostics() {
    assert_eq!(
        analyze(
            "😀 \t",
            &Policy {
                style: None,
                final_newline: Some(true),
                bom: Some(true),
                notrail: true,
            }
        ),
        vec![
            Problem {
                line: 1,
                col: 1,
                code: "EV005",
                text: "byte order mark missing".to_string(),
            },
            Problem {
                line: 1,
                col: 2,
                code: "EV006",
                text: "trailing whitespace of 2 characters".to_string(),
            },
            Problem {
                line: 1,
                col: 4,
                code: "EV002",
                text: "missing final line terminator".to_string(),
            },
        ]
    );
}

#[test]
fn notrail_excludes_zero_width_space_unit_separator_and_bom() {
    assert_eq!(
        analyze(
            "a\u{200b}\nb\u{001f}\nc\u{feff}",
            &Policy {
                style: None,
                final_newline: None,
                bom: None,
                notrail: true,
            }
        ),
        Vec::<Problem>::new()
    );
}

#[test]
fn same_coordinate_diagnostics_sort_by_code() {
    assert_eq!(
        analyze(
            "\r\n",
            &Policy {
                style: Some("lf".to_string()),
                final_newline: Some(false),
                bom: Some(true),
                notrail: true,
            }
        ),
        vec![
            Problem {
                line: 1,
                col: 1,
                code: "EV001",
                text: "CRLF terminator, expected LF".to_string(),
            },
            Problem {
                line: 1,
                col: 1,
                code: "EV003",
                text: "unexpected final line terminator".to_string(),
            },
            Problem {
                line: 1,
                col: 1,
                code: "EV005",
                text: "byte order mark missing".to_string(),
            },
        ]
    );
}

#[test]
fn every_supported_style_matches_its_python_terminator() {
    for (style, text) in [
        ("lf", "\n"),
        ("crlf", "\r\n"),
        ("cr", "\r"),
        ("nel", "\u{0085}"),
        ("ls", "\u{2028}"),
        ("ps", "\u{2029}"),
    ] {
        assert_eq!(
            analyze(
                text,
                &Policy {
                    style: Some(style.to_string()),
                    final_newline: None,
                    bom: None,
                    notrail: false,
                }
            ),
            Vec::<Problem>::new()
        );
    }
}
