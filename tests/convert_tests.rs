use eolscope::cmd_convert;
use eolscope::convert::convert;
use eolscope::error::EsError;
use eolscope::profile::Profile;
use eolscope::style::Style;

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
fn conversion_ac001_replaces_each_recognized_terminator() {
    for (text, from, to, final_newline, strip_bom, expected) in [
        (
            "a\nb\n",
            Profile::Lf,
            Style::Crlf,
            false,
            false,
            "a\r\nb\r\n",
        ),
        (
            "a\r\nb\rc",
            Profile::Universal,
            Style::Lf,
            false,
            false,
            "a\nb\nc",
        ),
        (
            "a\u{2028}b\u{2029}c\u{0085}d",
            Profile::Python,
            Style::Lf,
            false,
            false,
            "a\nb\nc\nd",
        ),
        (
            "a\u{001c}b\u{001d}c\u{001e}d\n",
            Profile::Python,
            Style::Lf,
            false,
            false,
            "a\nb\nc\nd\n",
        ),
        (
            "a\nb\n",
            Profile::Lf,
            Style::Ls,
            false,
            false,
            "a\u{2028}b\u{2028}",
        ),
        (
            "a\nb",
            Profile::Lf,
            Style::Ps,
            true,
            false,
            "a\u{2029}b\u{2029}",
        ),
        (
            "\u{FEFF}a\n",
            Profile::Lf,
            Style::Nel,
            false,
            false,
            "\u{FEFF}a\u{0085}",
        ),
        ("a\n", Profile::Unicode, Style::Cr, false, false, "a\r"),
    ] {
        assert_eq!(convert(text, from, to, final_newline, strip_bom), expected);
    }
}

#[test]
fn conversion_ac002_preserves_profile_sensitive_content() {
    for (text, from, to, final_newline, strip_bom, expected) in [
        (
            "a\u{2028}b\n",
            Profile::Lf,
            Style::Crlf,
            false,
            false,
            "a\u{2028}b\r\n",
        ),
        ("a\r", Profile::Go, Style::Lf, false, false, "a"),
        ("a\r", Profile::Rust, Style::Lf, false, false, "a\r"),
        ("a\r\n", Profile::Lf, Style::Lf, false, false, "a\r\n"),
        ("a\r\n", Profile::Rust, Style::Lf, false, false, "a\n"),
        ("a\r\n", Profile::Go, Style::Lf, false, false, "a\n"),
        (
            "a\u{001c}b\u{001d}c\u{001e}d",
            Profile::Unicode,
            Style::Lf,
            false,
            false,
            "a\u{001c}b\u{001d}c\u{001e}d",
        ),
        (
            "😀 \t \u{200B}\n",
            Profile::Python,
            Style::Crlf,
            false,
            false,
            "😀 \t \u{200B}\r\n",
        ),
    ] {
        assert_eq!(convert(text, from, to, final_newline, strip_bom), expected);
    }
}

#[test]
fn conversion_ac003_applies_final_newline_and_one_bom_removal() {
    for (text, from, to, final_newline, strip_bom, expected) in [
        ("", Profile::Unicode, Style::Cr, true, false, ""),
        ("\u{FEFF}", Profile::Lf, Style::Crlf, true, true, ""),
        (
            "\u{FEFF}a\u{2028}b\u{2029}",
            Profile::Js,
            Style::Lf,
            false,
            true,
            "a\nb\n",
        ),
        (
            "\u{FEFF}a\u{0085}b",
            Profile::Lf,
            Style::Crlf,
            true,
            true,
            "a\u{0085}b\r\n",
        ),
        (
            "\u{FEFF}\u{FEFF}a",
            Profile::Lf,
            Style::Lf,
            false,
            true,
            "\u{FEFF}a",
        ),
        (
            "a\u{FEFF}",
            Profile::Lf,
            Style::Lf,
            true,
            true,
            "a\u{FEFF}\n",
        ),
        ("\r", Profile::Go, Style::Lf, true, false, "\n"),
        ("a\n\n", Profile::Lf, Style::Crlf, true, false, "a\r\n\r\n"),
    ] {
        assert_eq!(convert(text, from, to, final_newline, strip_bom), expected);
    }
}

#[test]
fn convert_command_reads_file_with_repeated_interspersed_flags() {
    let io = FakeIo {
        files: vec![("t.txt".to_string(), "\u{FEFF}a\nb".as_bytes().to_vec())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_convert::run(
            &[
                "lf".to_string(),
                "crlf".to_string(),
                "t.txt".to_string(),
                "--strip-bom".to_string(),
                "--final-newline".to_string(),
                "--strip-bom".to_string(),
            ],
            &io,
        ),
        Ok(0)
    );
    assert_eq!(io.out.borrow().as_str(), "a\r\nb\r\n");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn convert_command_reads_go_profile_from_stdin() {
    let io = FakeIo {
        files: Vec::new(),
        stdin: b"a\r\nb\r".to_vec(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_convert::run(&["go".to_string(), "lf".to_string(), "-".to_string()], &io,),
        Ok(0)
    );
    assert_eq!(io.out.borrow().as_str(), "a\nb");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn convert_command_writes_empty_file_output() {
    let io = FakeIo {
        files: vec![("empty".to_string(), Vec::new())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_convert::run(
            &["lf".to_string(), "crlf".to_string(), "empty".to_string()],
            &io,
        ),
        Ok(0)
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn convert_command_rejects_unknown_options_first() {
    let io = FakeIo {
        files: Vec::new(),
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_convert::run(
            &[
                "--oops".to_string(),
                "LF".to_string(),
                "tab".to_string(),
                "missing.txt".to_string(),
                "--bad".to_string(),
            ],
            &io,
        ),
        Err(EsError("unknown option \"--oops\"".to_string()))
    );
    assert_eq!(
        cmd_convert::run(&["--help".to_string()], &io),
        Err(EsError("unknown option \"--help\"".to_string()))
    );
    assert_eq!(
        cmd_convert::run(&["--final-newline=true".to_string()], &io),
        Err(EsError(
            "unknown option \"--final-newline=true\"".to_string()
        ))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn convert_command_checks_exact_arity_and_usage() {
    let io = FakeIo {
        files: Vec::new(),
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_convert::USAGE,
        "convert [--final-newline] [--strip-bom] FROM TO FILE"
    );
    assert_eq!(
        cmd_convert::run(&["lf".to_string(), "crlf".to_string()], &io),
        Err(EsError(
            "usage: convert [--final-newline] [--strip-bom] FROM TO FILE".to_string()
        ))
    );
    assert_eq!(
        cmd_convert::run(
            &[
                "lf".to_string(),
                "crlf".to_string(),
                "a".to_string(),
                "b".to_string(),
            ],
            &io,
        ),
        Err(EsError(
            "usage: convert [--final-newline] [--strip-bom] FROM TO FILE".to_string()
        ))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn convert_command_validates_profile_then_style_before_file() {
    let io = FakeIo {
        files: Vec::new(),
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_convert::run(
            &[
                "LF".to_string(),
                "tab".to_string(),
                "missing.txt".to_string()
            ],
            &io,
        ),
        Err(EsError("unknown profile \"LF\"".to_string()))
    );
    assert_eq!(
        cmd_convert::run(
            &[
                "lf".to_string(),
                "tab".to_string(),
                "missing.txt".to_string()
            ],
            &io,
        ),
        Err(EsError("unknown style \"tab\"".to_string()))
    );
    assert_eq!(
        cmd_convert::run(
            &[
                "lf".to_string(),
                "CRLF".to_string(),
                "missing.txt".to_string()
            ],
            &io,
        ),
        Err(EsError("unknown style \"CRLF\"".to_string()))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn convert_command_reports_missing_unicode_path() {
    let io = FakeIo {
        files: Vec::new(),
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_convert::run(
            &["lf".to_string(), "lf".to_string(), "café😀.txt".to_string()],
            &io,
        ),
        Err(EsError("caf??.txt: cannot read the file".to_string()))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn convert_command_rejects_invalid_file_utf8() {
    let io = FakeIo {
        files: vec![("t.txt".to_string(), vec![97, 255, 10])],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_convert::run(
            &["lf".to_string(), "lf".to_string(), "t.txt".to_string()],
            &io,
        ),
        Err(EsError("t.txt: input is not valid UTF-8".to_string()))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn convert_command_rejects_invalid_stdin_utf8() {
    let io = FakeIo {
        files: Vec::new(),
        stdin: vec![97, 255, 10],
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_convert::run(&["lf".to_string(), "lf".to_string(), "-".to_string()], &io,),
        Err(EsError("<stdin>: input is not valid UTF-8".to_string()))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}
