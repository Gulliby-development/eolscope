use eolscope::kind::Kind;
use eolscope::profile::Profile;
use eolscope::splitter::{Line, split};

#[test]
fn splitting_empty_and_plain_text() {
    assert_eq!(split("", Profile::Python), Vec::<Line>::new());
    assert_eq!(
        split("a", Profile::Lf),
        vec![Line {
            content: "a".to_string(),
            kind: None,
            start: 0,
            end: 1,
        }]
    );
}

#[test]
fn splitting_consecutive_final_lf_has_no_phantom_line() {
    assert_eq!(
        split("a\n\n", Profile::Lf),
        vec![
            Line {
                content: "a".to_string(),
                kind: Some(Kind::Lf),
                start: 0,
                end: 2,
            },
            Line {
                content: String::new(),
                kind: Some(Kind::Lf),
                start: 2,
                end: 3,
            },
        ]
    );
}

#[test]
fn splitting_single_lf_and_unterminated_tail() {
    assert_eq!(
        split("\n", Profile::Go),
        vec![Line {
            content: String::new(),
            kind: Some(Kind::Lf),
            start: 0,
            end: 1,
        }]
    );
    assert_eq!(
        split("a\nb", Profile::Rust),
        vec![
            Line {
                content: "a".to_string(),
                kind: Some(Kind::Lf),
                start: 0,
                end: 2,
            },
            Line {
                content: "b".to_string(),
                kind: None,
                start: 2,
                end: 3,
            },
        ]
    );
}

#[test]
fn lf_preserves_cr_before_lf_and_terminal_cr() {
    assert_eq!(
        split("a\r\n", Profile::Lf),
        vec![Line {
            content: "a\r".to_string(),
            kind: Some(Kind::Lf),
            start: 0,
            end: 3,
        }]
    );
    assert_eq!(
        split("a\r", Profile::Lf),
        vec![Line {
            content: "a\r".to_string(),
            kind: None,
            start: 0,
            end: 2,
        }]
    );
}

#[test]
fn go_and_rust_coalesce_crlf_and_keep_scalar_spans() {
    assert_eq!(
        split("a\r\n", Profile::Go),
        vec![Line {
            content: "a".to_string(),
            kind: Some(Kind::Crlf),
            start: 0,
            end: 3,
        }]
    );
    assert_eq!(
        split("a\r\n", Profile::Rust),
        vec![Line {
            content: "a".to_string(),
            kind: Some(Kind::Crlf),
            start: 0,
            end: 3,
        }]
    );
}

#[test]
fn universal_coalesces_crlf_and_splits_standalone_cr() {
    // Input table case: ["a\r\nb\rc","universal"]
    assert_eq!(
        split("a\r\nb\rc", Profile::Universal),
        vec![
            Line {
                content: "a".to_string(),
                kind: Some(Kind::Crlf),
                start: 0,
                end: 3,
            },
            Line {
                content: "b".to_string(),
                kind: Some(Kind::Cr),
                start: 3,
                end: 5,
            },
            Line {
                content: "c".to_string(),
                kind: None,
                start: 5,
                end: 6,
            },
        ]
    );
}

#[test]
fn repeated_cr_is_distinct_for_go_and_python() {
    assert_eq!(
        split("a\r\r\n", Profile::Go),
        vec![Line {
            content: "a\r".to_string(),
            kind: Some(Kind::Crlf),
            start: 0,
            end: 4,
        }]
    );
    assert_eq!(
        split("a\r\r\n", Profile::Python),
        vec![
            Line {
                content: "a".to_string(),
                kind: Some(Kind::Cr),
                start: 0,
                end: 2,
            },
            Line {
                content: String::new(),
                kind: Some(Kind::Crlf),
                start: 2,
                end: 4,
            },
        ]
    );
}

#[test]
fn terminal_cr_behavior_in_lf_rust_and_go() {
    assert_eq!(
        split("a\r", Profile::Rust),
        vec![Line {
            content: "a\r".to_string(),
            kind: None,
            start: 0,
            end: 2,
        }]
    );
    assert_eq!(
        split("a\r", Profile::Go),
        vec![Line {
            content: "a".to_string(),
            kind: None,
            start: 0,
            end: 2,
        }]
    );
    assert_eq!(
        split("\r", Profile::Go),
        vec![Line {
            content: String::new(),
            kind: None,
            start: 0,
            end: 1,
        }]
    );
    assert_eq!(
        split("\r\r", Profile::Go),
        vec![Line {
            content: "\r".to_string(),
            kind: None,
            start: 0,
            end: 2,
        }]
    );
}

#[test]
fn rust_splits_adjacent_lf_and_crlf() {
    assert_eq!(
        split("\n\r\n", Profile::Rust),
        vec![
            Line {
                content: String::new(),
                kind: Some(Kind::Lf),
                start: 0,
                end: 1,
            },
            Line {
                content: String::new(),
                kind: Some(Kind::Crlf),
                start: 1,
                end: 3,
            },
        ]
    );
}

#[test]
fn bom_astral_and_bmp_scalars_count_once() {
    // Input table case: ["﻿😀\r\né x","python"]
    assert_eq!(
        split("\u{feff}😀\r\né\u{2028}x", Profile::Python),
        vec![
            Line {
                content: "\u{feff}😀".to_string(),
                kind: Some(Kind::Crlf),
                start: 0,
                end: 4,
            },
            Line {
                content: "é".to_string(),
                kind: Some(Kind::Ls),
                start: 4,
                end: 6,
            },
            Line {
                content: "x".to_string(),
                kind: None,
                start: 6,
                end: 7,
            },
        ]
    );
    assert_eq!(
        split("\u{feff}", Profile::Go),
        vec![Line {
            content: "\u{feff}".to_string(),
            kind: None,
            start: 0,
            end: 1,
        }]
    );
}

#[test]
fn python_and_unicode_differ_on_fs_gs_rs() {
    assert_eq!(
        split("a\u{001c}b\u{001d}c\u{001e}d", Profile::Python),
        vec![
            Line {
                content: "a".to_string(),
                kind: Some(Kind::Fs),
                start: 0,
                end: 2,
            },
            Line {
                content: "b".to_string(),
                kind: Some(Kind::Gs),
                start: 2,
                end: 4,
            },
            Line {
                content: "c".to_string(),
                kind: Some(Kind::Rs),
                start: 4,
                end: 6,
            },
            Line {
                content: "d".to_string(),
                kind: None,
                start: 6,
                end: 7,
            },
        ]
    );
    assert_eq!(
        split("a\u{001c}b\u{001d}c\u{001e}d", Profile::Unicode),
        vec![Line {
            content: "a\u{001c}b\u{001d}c\u{001e}d".to_string(),
            kind: None,
            start: 0,
            end: 7,
        }]
    );
}

#[test]
fn js_unicode_and_lf_recognize_different_separators() {
    assert_eq!(
        split("a\u{2028}b\u{2029}c\u{0085}", Profile::Js),
        vec![
            Line {
                content: "a".to_string(),
                kind: Some(Kind::Ls),
                start: 0,
                end: 2,
            },
            Line {
                content: "b".to_string(),
                kind: Some(Kind::Ps),
                start: 2,
                end: 4,
            },
            Line {
                content: "c\u{0085}".to_string(),
                kind: None,
                start: 4,
                end: 6,
            },
        ]
    );
    assert_eq!(
        split("a\u{000b}b\u{000c}c\u{0085}", Profile::Unicode),
        vec![
            Line {
                content: "a".to_string(),
                kind: Some(Kind::Vt),
                start: 0,
                end: 2,
            },
            Line {
                content: "b".to_string(),
                kind: Some(Kind::Ff),
                start: 2,
                end: 4,
            },
            Line {
                content: "c".to_string(),
                kind: Some(Kind::Nel),
                start: 4,
                end: 6,
            },
        ]
    );
    assert_eq!(
        split("a\u{2028}b\u{0085}\n", Profile::Lf),
        vec![Line {
            content: "a\u{2028}b\u{0085}".to_string(),
            kind: Some(Kind::Lf),
            start: 0,
            end: 5,
        }]
    );
}
