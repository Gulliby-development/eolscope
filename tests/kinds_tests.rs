use eolscope::cmd_kinds;
use eolscope::cmd_where;
use eolscope::error::EsError;

use eolscope::profile::Profile;

struct FakeIo {
    files: Vec<(String, Vec<u8>)>,
    stdin: Vec<u8>,
    out: std::cell::RefCell<String>,
    err: std::cell::RefCell<String>,
}

impl eolscope::io::Io for FakeIo {
    fn read_stdin(&self) -> Result<Vec<u8>, String> {
        Ok(self.stdin.clone())
    }

    fn read_file(&self, path: &str) -> Result<Vec<u8>, String> {
        match self.files.iter().find(|(name, _)| name == path) {
            Some((_, data)) => Ok(data.clone()),
            None => Err("no such file".to_string()),
        }
    }

    fn out(&self, text: &str) {
        self.out.borrow_mut().push_str(text);
    }

    fn err(&self, text: &str) {
        self.err.borrow_mut().push_str(text);
    }
}

#[test]
fn empty_report_has_no_terminators_or_final_line() {
    assert_eq!(
        cmd_kinds::report("t.txt", ""),
        (
            vec![
                "t.txt:".to_string(),
                "  lines  0".to_string(),
                "  style  none".to_string(),
                "  final  -".to_string(),
                "  bom    no".to_string()
            ],
            false
        )
    );
}

#[test]
fn report_counts_lf_and_preserves_a_leading_bom() {
    assert_eq!(
        cmd_kinds::report("t.txt", "\u{feff}a\nb\nc"),
        (
            vec![
                "t.txt:".to_string(),
                "  lines  3".to_string(),
                "  LF     2".to_string(),
                "  style  LF".to_string(),
                "  final  no".to_string(),
                "  bom    yes".to_string()
            ],
            false
        )
    );
}

#[test]
fn report_counts_mixed_kinds_in_kind_order() {
    assert_eq!(
        cmd_kinds::report("t.txt", "a\nb\r\nc\u{0085}d\u{2028}e\u{2029}"),
        (
            vec![
                "t.txt:".to_string(),
                "  lines  5".to_string(),
                "  LF     1".to_string(),
                "  CRLF   1".to_string(),
                "  NEL    1".to_string(),
                "  LS     1".to_string(),
                "  PS     1".to_string(),
                "  style  mixed".to_string(),
                "  final  yes".to_string(),
                "  bom    no".to_string()
            ],
            true
        )
    );
}

#[test]
fn report_counts_control_kinds_and_does_not_treat_an_interior_bom_as_a_prefix() {
    assert_eq!(
        cmd_kinds::report("t.txt", "\r\u{000b}\u{000c}\u{001c}\u{001d}\u{001e}"),
        (
            vec![
                "t.txt:".to_string(),
                "  lines  6".to_string(),
                "  CR     1".to_string(),
                "  VT     1".to_string(),
                "  FF     1".to_string(),
                "  FS     1".to_string(),
                "  GS     1".to_string(),
                "  RS     1".to_string(),
                "  style  mixed".to_string(),
                "  final  yes".to_string(),
                "  bom    no".to_string()
            ],
            true
        )
    );
    assert_eq!(
        cmd_kinds::report("t.txt", "x\u{feff}"),
        (
            vec![
                "t.txt:".to_string(),
                "  lines  1".to_string(),
                "  style  none".to_string(),
                "  final  no".to_string(),
                "  bom    no".to_string()
            ],
            false
        )
    );
}

#[test]
fn kinds_keeps_repeated_files_in_order_and_mixes_styles_per_file() {
    let io = FakeIo {
        files: vec![
            ("a.txt".to_string(), b"a\n".to_vec()),
            ("b.txt".to_string(), b"b\r".to_vec()),
        ],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_kinds::run(
            &[
                "a.txt".to_string(),
                "b.txt".to_string(),
                "a.txt".to_string()
            ],
            &io
        ),
        Ok(0)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        "a.txt:\n  lines  1\n  LF     1\n  style  LF\n  final  yes\n  bom    no\nb.txt:\n  lines  1\n  CR     1\n  style  CR\n  final  yes\n  bom    no\na.txt:\n  lines  1\n  LF     1\n  style  LF\n  final  yes\n  bom    no\n"
    );
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn kinds_returns_one_for_a_mixed_file() {
    let io = FakeIo {
        files: vec![("mix.txt".to_string(), b"a\nb\r".to_vec())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(cmd_kinds::run(&["mix.txt".to_string()], &io), Ok(1));
    assert_eq!(
        io.out.borrow().as_str(),
        "mix.txt:\n  lines  2\n  LF     1\n  CR     1\n  style  mixed\n  final  yes\n  bom    no\n"
    );
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn kinds_buffers_errors_and_rejects_duplicate_stdin_without_output() {
    let missing_io = FakeIo {
        files: vec![("a.txt".to_string(), b"a\n".to_vec())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_kinds::run(
            &["a.txt".to_string(), "missing.txt".to_string()],
            &missing_io
        ),
        Err(EsError("missing.txt: cannot read the file".to_string()))
    );
    assert_eq!(missing_io.out.borrow().as_str(), "");
    assert_eq!(missing_io.err.borrow().as_str(), "");

    let stdin_io = FakeIo {
        files: vec![],
        stdin: b"a\n".to_vec(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_kinds::run(&["-".to_string(), "-".to_string()], &stdin_io),
        Err(EsError("standard input is named twice".to_string()))
    );
    assert_eq!(stdin_io.out.borrow().as_str(), "");
    assert_eq!(stdin_io.err.borrow().as_str(), "");
}

#[test]
fn locate_matches_the_seven_profile_examples_and_counts_unicode_scalars() {
    let rows = [
        ("lf", 5, 1, 6, "LF"),
        ("go", 5, 1, 5, "CRLF"),
        ("rust", 4, 1, 5, "CRLF"),
        ("universal", 5, 1, 5, "CRLF"),
        ("js", 4, 2, 2, "CRLF"),
        ("python", 5, 2, 2, "CRLF"),
        ("unicode", 5, 2, 2, "CRLF"),
    ];
    for (name, offset, line, column, at) in rows {
        let profile = Profile::parse(name);
        assert!(profile.is_some());
        if let Some(profile) = profile {
            assert_eq!(
                cmd_where::locate("ab\u{2028}c\r\nd\u{0085}e", profile, offset),
                (line, column, at.to_string())
            );
        }
    }
    let profile = Profile::parse("python");
    assert!(profile.is_some());
    if let Some(profile) = profile {
        assert_eq!(
            cmd_where::locate("\u{feff}😀x", profile, 2),
            (1, 3, "text".to_string())
        );
    }
}

#[test]
fn locate_reports_go_dropped_cr_ordinary_cr_and_each_eof_shape() {
    let rows = [
        ("ab\r", "go", 2, 1, 3, "CR"),
        ("ab\r", "lf", 2, 1, 3, "text"),
        ("ab\r", "universal", 2, 1, 3, "CR"),
        ("ab\r", "go", 3, 1, 3, "end"),
        ("ab\r", "rust", 3, 1, 4, "end"),
        ("ab\r", "python", 3, 2, 1, "end"),
        ("", "unicode", 0, 1, 1, "end"),
        ("a\n", "js", 2, 2, 1, "end"),
    ];
    for (text, name, offset, line, column, at) in rows {
        let profile = Profile::parse(name);
        assert!(profile.is_some());
        if let Some(profile) = profile {
            assert_eq!(
                cmd_where::locate(text, profile, offset),
                (line, column, at.to_string())
            );
        }
    }
}

#[test]
fn where_renders_the_exact_eof_table_for_ab_cr() {
    let io = FakeIo {
        files: vec![("t.txt".to_string(), b"ab\r".to_vec())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_where::run(&["3".to_string(), "t.txt".to_string()], &io),
        Ok(0)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        "PROFILE    LINE  COL  AT\nlf         1     4    end\ngo         1     3    end\nrust       1     4    end\nuniversal  2     1    end\njs         2     1    end\npython     2     1    end\nunicode    2     1    end\n"
    );
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn where_rejects_noncanonical_offsets_before_reading_files() {
    let io = FakeIo {
        files: vec![],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    let cases = [
        ("", "invalid offset \"\""),
        ("007", "invalid offset \"007\""),
        ("1234567890", "invalid offset \"1234567890\""),
        ("+1", "invalid offset \"+1\""),
        ("-1", "invalid offset \"-1\""),
        ("1x", "invalid offset \"1x\""),
        ("１", "invalid offset \"?\""),
    ];
    for (offset, expected) in cases {
        assert_eq!(
            cmd_where::run(&[offset.to_string(), "missing.txt".to_string()], &io),
            Err(EsError(expected.to_string()))
        );
        assert_eq!(io.out.borrow().as_str(), "");
        assert_eq!(io.err.borrow().as_str(), "");
    }
}

#[test]
fn where_checks_range_pluralization_and_accepts_zero_on_empty_text() {
    let io = FakeIo {
        files: vec![
            ("t.txt".to_string(), b"ab".to_vec()),
            ("one.txt".to_string(), b"a".to_vec()),
            ("empty.txt".to_string(), Vec::new()),
        ],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_where::run(&["9".to_string(), "t.txt".to_string()], &io),
        Err(EsError(
            "offset 9 is beyond the end (2 characters)".to_string()
        ))
    );
    assert_eq!(
        cmd_where::run(&["3".to_string(), "one.txt".to_string()], &io),
        Err(EsError(
            "offset 3 is beyond the end (1 character)".to_string()
        ))
    );
    assert_eq!(
        cmd_where::run(&["999999999".to_string(), "empty.txt".to_string()], &io),
        Err(EsError(
            "offset 999999999 is beyond the end (0 characters)".to_string()
        ))
    );
    let boundary_io = FakeIo {
        files: vec![("t.txt".to_string(), b"ab".to_vec())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_where::run(&["2".to_string(), "t.txt".to_string()], &boundary_io),
        Ok(0)
    );
    assert_eq!(
        boundary_io.out.borrow().as_str(),
        "PROFILE    LINE  COL  AT\nlf         1     3    end\ngo         1     3    end\nrust       1     3    end\nuniversal  1     3    end\njs         1     3    end\npython     1     3    end\nunicode    1     3    end\n"
    );
    assert_eq!(boundary_io.err.borrow().as_str(), "");

    assert_eq!(
        cmd_where::run(&["0".to_string(), "empty.txt".to_string()], &io),
        Ok(0)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        "PROFILE    LINE  COL  AT\nlf         1     1    end\ngo         1     1    end\nrust       1     1    end\nuniversal  1     1    end\njs         1     1    end\npython     1     1    end\nunicode    1     1    end\n"
    );
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn where_reads_before_range_and_rejects_bad_utf8_and_sanitizes_paths() {
    let io = FakeIo {
        files: vec![("bad.txt".to_string(), vec![97, 255, 10])],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_where::run(&["9".to_string(), "missing.txt".to_string()], &io),
        Err(EsError("missing.txt: cannot read the file".to_string()))
    );
    assert_eq!(
        cmd_where::run(&["0".to_string(), "bad.txt".to_string()], &io),
        Err(EsError("bad.txt: input is not valid UTF-8".to_string()))
    );
    assert_eq!(
        cmd_kinds::run(&["café😀.txt".to_string()], &io),
        Err(EsError("caf??.txt: cannot read the file".to_string()))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn commands_have_exact_usage_and_unknown_options_precede_arity() {
    assert_eq!(cmd_kinds::USAGE, "kinds FILE...");
    assert_eq!(cmd_where::USAGE, "where OFFSET FILE");
    let io = FakeIo {
        files: vec![],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_kinds::run(&[], &io),
        Err(EsError("usage: kinds FILE...".to_string()))
    );
    assert_eq!(
        cmd_where::run(&[], &io),
        Err(EsError("usage: where OFFSET FILE".to_string()))
    );
    assert_eq!(
        cmd_where::run(&["1".to_string()], &io),
        Err(EsError("usage: where OFFSET FILE".to_string()))
    );
    assert_eq!(
        cmd_where::run(&["1".to_string(), "a".to_string(), "b".to_string()], &io),
        Err(EsError("usage: where OFFSET FILE".to_string()))
    );
    assert_eq!(
        cmd_kinds::run(&["--oops".to_string()], &io),
        Err(EsError("unknown option \"--oops\"".to_string()))
    );
    assert_eq!(
        cmd_where::run(&["--oops".to_string()], &io),
        Err(EsError("unknown option \"--oops\"".to_string()))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}
