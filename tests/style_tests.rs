use eolscope::bom::{has_bom, strip_bom};
use eolscope::style::Style;

#[test]
fn all_styles_are_in_contract_order() {
    assert_eq!(
        Style::ALL,
        [
            Style::Lf,
            Style::Crlf,
            Style::Cr,
            Style::Nel,
            Style::Ls,
            Style::Ps,
        ]
    );
}

#[test]
fn style_names_match_every_variant() {
    for (style, name) in [
        (Style::Lf, "lf"),
        (Style::Crlf, "crlf"),
        (Style::Cr, "cr"),
        (Style::Nel, "nel"),
        (Style::Ls, "ls"),
        (Style::Ps, "ps"),
    ] {
        assert_eq!(style.name(), name);
    }
}

#[test]
fn parses_every_valid_style_name() {
    for (name, style) in [
        ("lf", Style::Lf),
        ("crlf", Style::Crlf),
        ("cr", Style::Cr),
        ("nel", Style::Nel),
        ("ls", Style::Ls),
        ("ps", Style::Ps),
    ] {
        assert_eq!(Style::parse(name), Some(style));
    }
}

#[test]
fn style_texts_are_exact_terminators() {
    for (style, text) in [
        (Style::Lf, "\n"),
        (Style::Crlf, "\r\n"),
        (Style::Cr, "\r"),
        (Style::Nel, "\u{0085}"),
        (Style::Ls, "\u{2028}"),
        (Style::Ps, "\u{2029}"),
    ] {
        assert_eq!(style.text(), text);
    }
}

#[test]
fn style_kind_names_match_every_variant() {
    for (style, kind_name) in [
        (Style::Lf, "LF"),
        (Style::Crlf, "CRLF"),
        (Style::Cr, "CR"),
        (Style::Nel, "NEL"),
        (Style::Ls, "LS"),
        (Style::Ps, "PS"),
    ] {
        assert_eq!(style.kind_name(), kind_name);
    }
}

#[test]
fn parse_rejects_uppercase_and_mixed_case_names() {
    assert_eq!(Style::parse("LF"), None);
    assert_eq!(Style::parse("CRLF"), None);
    assert_eq!(Style::parse("Cr"), None);
    assert_eq!(Style::parse("nEl"), None);
    assert_eq!(Style::parse("Ls"), None);
    assert_eq!(Style::parse("pS"), None);
}

#[test]
fn parse_rejects_empty_unknown_and_padded_names() {
    assert_eq!(Style::parse(""), None);
    assert_eq!(Style::parse("unknown"), None);
    assert_eq!(Style::parse(" lf"), None);
    assert_eq!(Style::parse("ps "), None);
}

#[test]
fn empty_and_ordinary_text_have_no_bom() {
    assert!(!has_bom(""));
    assert_eq!(strip_bom(""), "");
    assert!(!has_bom("plain text"));
    assert_eq!(strip_bom("plain text"), "plain text");
}

#[test]
fn bom_only_and_bom_with_multibyte_suffix_are_handled() {
    assert!(has_bom("\u{FEFF}"));
    assert_eq!(strip_bom("\u{FEFF}"), "");
    assert!(has_bom("\u{FEFF}é😀"));
    assert_eq!(strip_bom("\u{FEFF}é😀"), "é😀");
}

#[test]
fn repeated_leading_bom_removes_exactly_one() {
    assert!(has_bom("\u{FEFF}\u{FEFF}text"));
    assert_eq!(strip_bom("\u{FEFF}\u{FEFF}text"), "\u{FEFF}text");
}

#[test]
fn interior_trailing_and_other_unicode_prefixes_are_unchanged() {
    assert!(!has_bom("a\u{FEFF}b"));
    assert_eq!(strip_bom("a\u{FEFF}b"), "a\u{FEFF}b");
    assert!(!has_bom("text\u{FEFF}"));
    assert_eq!(strip_bom("text\u{FEFF}"), "text\u{FEFF}");
    assert!(!has_bom("é😀text"));
    assert_eq!(strip_bom("é😀text"), "é😀text");
}
