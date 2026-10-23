use eolscope::kind::Kind;
use eolscope::profile::Profile;

#[test]
fn kind_all_is_in_declared_order_and_ord_agrees() {
    assert_eq!(
        Kind::ALL,
        [
            Kind::Lf,
            Kind::Crlf,
            Kind::Cr,
            Kind::Vt,
            Kind::Ff,
            Kind::Fs,
            Kind::Gs,
            Kind::Rs,
            Kind::Nel,
            Kind::Ls,
            Kind::Ps,
        ]
    );
    assert!(Kind::ALL.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn kind_names_match_all_variants() {
    assert_eq!(
        Kind::ALL.map(|kind| kind.name()),
        [
            "LF", "CRLF", "CR", "VT", "FF", "FS", "GS", "RS", "NEL", "LS", "PS"
        ]
    );
}

#[test]
fn kind_classifies_lf_cr_vt_and_ff() {
    let cases = [
        ('\n', Some(Kind::Lf)),
        ('\r', Some(Kind::Cr)),
        ('\u{000b}', Some(Kind::Vt)),
        ('\u{000c}', Some(Kind::Ff)),
    ];
    for (character, expected) in cases {
        assert_eq!(Kind::of_char(character), expected);
    }
}

#[test]
fn kind_classifies_fs_gs_rs_nel_ls_and_ps() {
    let cases = [
        ('\u{001c}', Some(Kind::Fs)),
        ('\u{001d}', Some(Kind::Gs)),
        ('\u{001e}', Some(Kind::Rs)),
        ('\u{0085}', Some(Kind::Nel)),
        ('\u{2028}', Some(Kind::Ls)),
        ('\u{2029}', Some(Kind::Ps)),
    ];
    for (character, expected) in cases {
        assert_eq!(Kind::of_char(character), expected);
    }
}

#[test]
fn kind_rejects_non_terminators() {
    let cases = ['\u{001f}', '\u{feff}', '😀', ' ', '\t'];
    for character in cases {
        assert_eq!(Kind::of_char(character), None);
    }
}

#[test]
fn profile_all_and_names_match_declaration_order() {
    assert_eq!(
        Profile::ALL,
        [
            Profile::Lf,
            Profile::Go,
            Profile::Rust,
            Profile::Universal,
            Profile::Js,
            Profile::Python,
            Profile::Unicode,
        ]
    );
    assert_eq!(
        Profile::ALL.map(|profile| profile.name()),
        ["lf", "go", "rust", "universal", "js", "python", "unicode"]
    );
}

#[test]
fn profile_parses_each_exact_lowercase_name() {
    let cases = [
        ("lf", Profile::Lf),
        ("go", Profile::Go),
        ("rust", Profile::Rust),
        ("universal", Profile::Universal),
        ("js", Profile::Js),
        ("python", Profile::Python),
        ("unicode", Profile::Unicode),
    ];
    for (name, expected) in cases {
        assert_eq!(Profile::parse(name), Some(expected));
    }
}

#[test]
fn profile_rejects_case_and_whitespace_variants() {
    let names = ["LF", "", " rust", "rust ", "unknown"];
    for name in names {
        assert_eq!(Profile::parse(name), None);
    }
}

#[test]
fn lf_and_cr_membership_is_exact() {
    let cases: [(char, &[&str]); 2] = [
        (
            '\n',
            &["lf", "go", "rust", "universal", "js", "python", "unicode"][..],
        ),
        ('\r', &["universal", "js", "python", "unicode"][..]),
    ];
    for (character, expected) in cases {
        let actual: Vec<&str> = Profile::ALL
            .into_iter()
            .filter(|profile| profile.ends_line(character))
            .map(|profile| profile.name())
            .collect();
        assert_eq!(actual.as_slice(), expected);
    }
}

#[test]
fn vt_ff_and_nel_membership_is_exact() {
    let cases: [(char, &[&str]); 3] = [
        ('\u{000b}', &["python", "unicode"][..]),
        ('\u{000c}', &["python", "unicode"][..]),
        ('\u{0085}', &["python", "unicode"][..]),
    ];
    for (character, expected) in cases {
        let actual: Vec<&str> = Profile::ALL
            .into_iter()
            .filter(|profile| profile.ends_line(character))
            .map(|profile| profile.name())
            .collect();
        assert_eq!(actual.as_slice(), expected);
    }
}

#[test]
fn fs_gs_and_rs_are_python_only() {
    let cases = [
        ('\u{001c}', &["python"][..]),
        ('\u{001d}', &["python"][..]),
        ('\u{001e}', &["python"][..]),
    ];
    for (character, expected) in cases {
        let actual: Vec<&str> = Profile::ALL
            .into_iter()
            .filter(|profile| profile.ends_line(character))
            .map(|profile| profile.name())
            .collect();
        assert_eq!(actual.as_slice(), expected);
    }
}

#[test]
fn ls_ps_and_other_special_scalars_have_exact_membership() {
    let cases: [(char, &[&str]); 7] = [
        ('\u{2028}', &["js", "python", "unicode"][..]),
        ('\u{2029}', &["js", "python", "unicode"][..]),
        ('\u{001f}', &[][..]),
        ('\t', &[][..]),
        ('\u{feff}', &[][..]),
        ('\u{00a0}', &[][..]),
        ('😀', &[][..]),
    ];
    for (character, expected) in cases {
        let actual: Vec<&str> = Profile::ALL
            .into_iter()
            .filter(|profile| profile.ends_line(character))
            .map(|profile| profile.name())
            .collect();
        assert_eq!(actual.as_slice(), expected);
    }
}
