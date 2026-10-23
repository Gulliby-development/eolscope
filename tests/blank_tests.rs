use eolscope::rule::Rule;
use eolscope::tail::trailing;

#[test]
fn rule_api_has_the_required_order_names_and_traits() {
    assert_eq!(
        Rule::ALL,
        [
            Rule::JsTrim,
            Rule::PyStrip,
            Rule::RsTrim,
            Rule::JavaTrim,
            Rule::JavaStrip,
        ]
    );
    assert_eq!(Rule::JsTrim.name(), "js-trim");
    assert_eq!(Rule::PyStrip.name(), "py-strip");
    assert_eq!(Rule::RsTrim.name(), "rs-trim");
    assert_eq!(Rule::JavaTrim.name(), "java-trim");
    assert_eq!(Rule::JavaStrip.name(), "java-strip");

    let copied = Rule::PyStrip;
    assert_eq!(copied, copied.clone());
    assert_eq!(format!("{copied:?}"), "PyStrip");
    assert_ne!(Rule::JsTrim, Rule::JavaTrim);
}

#[test]
fn strips_ascii_controls_first_group() {
    for (character, expected) in [
        ('\u{0000}', [false, false, false, true, false]),
        ('\u{0008}', [false, false, false, true, false]),
        ('\t', [true, true, true, true, true]),
        ('\n', [true, true, true, true, true]),
        ('\u{000B}', [true, true, true, true, true]),
        ('\u{000C}', [true, true, true, true, true]),
        ('\r', [true, true, true, true, true]),
        ('\u{000E}', [false, false, false, true, false]),
    ] {
        assert_eq!(Rule::ALL.map(|rule| rule.strips(character)), expected);
    }
}

#[test]
fn strips_ascii_controls_second_group() {
    for (character, expected) in [
        ('\u{001B}', [false, false, false, true, false]),
        ('\u{001C}', [false, true, false, true, true]),
        ('\u{001D}', [false, true, false, true, true]),
        ('\u{001E}', [false, true, false, true, true]),
        ('\u{001F}', [false, true, false, true, true]),
        (' ', [true, true, true, true, true]),
        ('!', [false, false, false, false, false]),
    ] {
        assert_eq!(Rule::ALL.map(|rule| rule.strips(character)), expected);
    }
}

#[test]
fn strips_distinguishing_unicode_first_group() {
    for (character, expected) in [
        ('\u{0085}', [false, true, true, false, false]),
        ('\u{00A0}', [true, true, true, false, false]),
        ('\u{2007}', [true, true, true, false, false]),
        ('\u{202F}', [true, true, true, false, false]),
        ('\u{FEFF}', [true, false, false, false, false]),
    ] {
        assert_eq!(Rule::ALL.map(|rule| rule.strips(character)), expected);
    }
}

#[test]
fn strips_distinguishing_unicode_second_group() {
    for (character, expected) in [
        ('\u{200B}', [false, false, false, false, false]),
        ('\u{2028}', [true, true, true, false, true]),
        ('\u{2029}', [true, true, true, false, true]),
        ('\u{3000}', [true, true, true, false, true]),
        ('😀', [false, false, false, false, false]),
    ] {
        assert_eq!(Rule::ALL.map(|rule| rule.strips(character)), expected);
    }
}

#[test]
fn strips_range_boundaries_first_group() {
    for (character, expected) in [
        ('\u{167F}', [false, false, false, false, false]),
        ('\u{1680}', [true, true, true, false, true]),
        ('\u{1681}', [false, false, false, false, false]),
        ('\u{1FFE}', [false, false, false, false, false]),
        ('\u{2000}', [true, true, true, false, true]),
        ('\u{2006}', [true, true, true, false, true]),
        ('\u{2008}', [true, true, true, false, true]),
    ] {
        assert_eq!(Rule::ALL.map(|rule| rule.strips(character)), expected);
    }
}

#[test]
fn strips_range_boundaries_second_group() {
    for (character, expected) in [
        ('\u{200A}', [true, true, true, false, true]),
        ('\u{2027}', [false, false, false, false, false]),
        ('\u{202A}', [false, false, false, false, false]),
        ('\u{202E}', [false, false, false, false, false]),
        ('\u{2030}', [false, false, false, false, false]),
        ('\u{205E}', [false, false, false, false, false]),
    ] {
        assert_eq!(Rule::ALL.map(|rule| rule.strips(character)), expected);
    }
}

#[test]
fn strips_range_boundaries_third_group() {
    for (character, expected) in [
        ('\u{205F}', [true, true, true, false, true]),
        ('\u{2060}', [false, false, false, false, false]),
        ('\u{2FFF}', [false, false, false, false, false]),
        ('\u{3001}', [false, false, false, false, false]),
        ('\u{FEFE}', [false, false, false, false, false]),
        ('\u{FF00}', [false, false, false, false, false]),
    ] {
        assert_eq!(Rule::ALL.map(|rule| rule.strips(character)), expected);
    }
}

#[test]
fn trailing_counts_empty_ascii_and_control_suffixes() {
    for (rule, text, expected) in [
        (Rule::JsTrim, "", 0),
        (Rule::PyStrip, "abc", 0),
        (Rule::RsTrim, " \t", 2),
        (Rule::JavaTrim, "a\u{0000}\u{001F}", 2),
        (Rule::JavaStrip, "a\u{001C}\u{001D}\u{001E}\u{001F}", 4),
        (Rule::JsTrim, "😀 　", 2),
    ] {
        assert_eq!(trailing(rule, text), expected);
    }
}

#[test]
fn trailing_stops_at_nonmembers_and_distinguishes_bom_position() {
    for (rule, text, expected) in [
        (Rule::PyStrip, "a \u{200B}\t", 1),
        (Rule::RsTrim, "a\t\u{200B}", 0),
        (Rule::JsTrim, "\u{FEFF}a ", 1),
        (Rule::JsTrim, "a\u{FEFF}", 1),
        (Rule::PyStrip, "a\u{FEFF}", 0),
        (Rule::RsTrim, "a\u{0085}", 1),
    ] {
        assert_eq!(trailing(rule, text), expected);
    }
}

#[test]
fn trailing_counts_rule_specific_and_multiscalar_suffixes() {
    for (rule, text, expected) in [
        (Rule::JsTrim, "a\u{0085}", 0),
        (Rule::JavaStrip, "a\t\u{00A0}", 0),
        (Rule::JavaTrim, "a\u{3000}", 0),
        (Rule::JavaStrip, "a\u{3000}", 1),
        (Rule::RsTrim, "a\r\n", 2),
    ] {
        assert_eq!(trailing(rule, text), expected);
    }
}
