use eolscope::error::EsError;
use eolscope::{cmd_agree, cmd_lines};

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
fn render_empty_and_lf_rows() {
    assert_eq!(cmd_lines::render("", Profile::Lf), vec!["none"]);
    assert_eq!(
        cmd_lines::render("a\n\n", Profile::Lf),
        vec![
            "LINE  END  CHARS  TEXT",
            "1     LF   1      \"a\"",
            "2     LF   0      \"\"",
        ]
    );
}

#[test]
fn render_go_and_python_profiles() {
    assert_eq!(
        cmd_lines::render("a\r\nb\r\n\rc\r", Profile::Go),
        vec![
            "LINE  END   CHARS  TEXT",
            "1     CRLF  1      \"a\"",
            "2     CRLF  1      \"b\"",
            "3     none  2      \"?c\"",
        ]
    );
    assert_eq!(
        cmd_lines::render(
            "one\u{2028}two\u{0085}three\u{001c}\n\u{feff}x",
            Profile::Python
        ),
        vec![
            "LINE  END   CHARS  TEXT",
            "1     LS    3      \"one\"",
            "2     NEL   3      \"two\"",
            "3     FS    5      \"three\"",
            "4     LF    0      \"\"",
            "5     none  2      \"?x\"",
        ]
    );
}

#[test]
fn render_scalar_count_bom_and_astral_character() {
    assert_eq!(
        cmd_lines::render(
            "one\u{2028}two\u{0085}three\u{001c}\n\u{feff}x",
            Profile::Lf
        ),
        vec![
            "LINE  END   CHARS  TEXT",
            "1     LF    14     \"one?two?three?\"",
            "2     none  2      \"?x\"",
        ]
    );
    assert_eq!(
        cmd_lines::render("\u{feff}😀\n", Profile::Rust),
        vec!["LINE  END  CHARS  TEXT", "1     LF   2      \"??\""]
    );
}

#[test]
fn render_terminal_cr_profile_differences() {
    assert_eq!(
        cmd_lines::render("a\r", Profile::Rust),
        vec!["LINE  END   CHARS  TEXT", "1     none  2      \"a?\""]
    );
    assert_eq!(
        cmd_lines::render("a\r", Profile::Go),
        vec!["LINE  END   CHARS  TEXT", "1     none  1      \"a\""]
    );
}

#[test]
fn summary_agreement_examples() {
    for (text, expected, agrees) in [
        (
            "",
            "t.txt: lf=0 go=0 rust=0 universal=0 js=0 python=0 unicode=0 agree",
            true,
        ),
        (
            "a\n",
            "t.txt: lf=1 go=1 rust=1 universal=1 js=1 python=1 unicode=1 agree",
            true,
        ),
        (
            "a\r\nb",
            "t.txt: lf=2 go=2 rust=2 universal=2 js=2 python=2 unicode=2 agree",
            true,
        ),
        (
            "a\r",
            "t.txt: lf=1 go=1 rust=1 universal=1 js=1 python=1 unicode=1 agree",
            true,
        ),
        (
            "a\rb",
            "t.txt: lf=1 go=1 rust=1 universal=2 js=2 python=2 unicode=2 differ",
            false,
        ),
        (
            "a\u{2028}b\n",
            "t.txt: lf=1 go=1 rust=1 universal=1 js=2 python=2 unicode=2 differ",
            false,
        ),
        (
            "a\u{2029}b\u{2028}",
            "t.txt: lf=1 go=1 rust=1 universal=1 js=2 python=2 unicode=2 differ",
            false,
        ),
        (
            "a\u{0085}b",
            "t.txt: lf=1 go=1 rust=1 universal=1 js=1 python=2 unicode=2 differ",
            false,
        ),
    ] {
        assert_eq!(
            cmd_agree::summary("t.txt", text),
            (expected.to_string(), agrees)
        );
    }
}

#[test]
fn summary_control_and_bom_examples() {
    for (text, expected, agrees) in [
        (
            "a\u{001c}b\u{001d}c\u{001e}d",
            "t.txt: lf=1 go=1 rust=1 universal=1 js=1 python=4 unicode=1 differ",
            false,
        ),
        (
            "\u{feff}a\n",
            "t.txt: lf=1 go=1 rust=1 universal=1 js=1 python=1 unicode=1 agree",
            true,
        ),
        (
            "a\r\r\n",
            "t.txt: lf=1 go=1 rust=1 universal=2 js=2 python=2 unicode=2 differ",
            false,
        ),
        (
            "\u{2028}\u{2029}\u{0085}",
            "t.txt: lf=1 go=1 rust=1 universal=1 js=3 python=3 unicode=3 differ",
            false,
        ),
    ] {
        assert_eq!(
            cmd_agree::summary("t.txt", text),
            (expected.to_string(), agrees)
        );
    }
}

#[test]
fn line_validation_and_option_precedence() {
    assert_eq!(cmd_lines::USAGE, "lines PROFILE FILE");
    let io = FakeIo {
        files: Vec::new(),
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cmd_lines::run(&["--oops".to_string()], &io),
        Err(EsError("unknown option \"--oops\"".to_string()))
    );
    assert_eq!(
        cmd_lines::run(&["lf".to_string()], &io),
        Err(EsError("usage: lines PROFILE FILE".to_string()))
    );
    assert_eq!(
        cmd_lines::run(
            &["lf".to_string(), "t.txt".to_string(), "extra".to_string()],
            &io
        ),
        Err(EsError("usage: lines PROFILE FILE".to_string()))
    );
    assert_eq!(
        cmd_lines::run(&["LF".to_string(), "missing.txt".to_string()], &io),
        Err(EsError("unknown profile \"LF\"".to_string()))
    );
    assert_eq!(
        cmd_lines::run(&["lf".to_string(), "missing.txt".to_string()], &io),
        Err(EsError("missing.txt: cannot read the file".to_string()))
    );
    assert_eq!(
        cmd_lines::run(
            &[
                "LF".to_string(),
                "missing.txt".to_string(),
                "--oops".to_string()
            ],
            &io
        ),
        Err(EsError("unknown option \"--oops\"".to_string()))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn agree_validation_and_option_precedence() {
    assert_eq!(cmd_agree::USAGE, "agree FILE...");
    let io = FakeIo {
        files: Vec::new(),
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cmd_agree::run(&[], &io),
        Err(EsError("usage: agree FILE...".to_string()))
    );
    assert_eq!(
        cmd_agree::run(&["--help".to_string()], &io),
        Err(EsError("unknown option \"--help\"".to_string()))
    );
    assert_eq!(
        cmd_agree::run(&["missing.txt".to_string(), "--oops".to_string()], &io),
        Err(EsError("unknown option \"--oops\"".to_string()))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn lines_command_reads_stdin_and_formats_output_once() {
    let io = FakeIo {
        files: Vec::new(),
        stdin: b"a\n".to_vec(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cmd_lines::run(&["lf".to_string(), "-".to_string()], &io),
        Ok(0)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        "LINE  END  CHARS  TEXT\n1     LF   1      \"a\"\n"
    );
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn agree_command_preserves_repeated_file_order() {
    let io = FakeIo {
        files: vec![("t.txt".to_string(), b"a\n".to_vec())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cmd_agree::run(&["t.txt".to_string(), "t.txt".to_string()], &io),
        Ok(0)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        "t.txt: lf=1 go=1 rust=1 universal=1 js=1 python=1 unicode=1 agree\nt.txt: lf=1 go=1 rust=1 universal=1 js=1 python=1 unicode=1 agree\n"
    );
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn agree_command_reports_mixed_files_and_stdin() {
    let files = FakeIo {
        files: vec![
            ("t.txt".to_string(), b"a\n".to_vec()),
            ("odd.txt".to_string(), b"a\rb".to_vec()),
        ],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cmd_agree::run(&["t.txt".to_string(), "odd.txt".to_string()], &files),
        Ok(1)
    );
    assert_eq!(
        files.out.borrow().as_str(),
        "t.txt: lf=1 go=1 rust=1 universal=1 js=1 python=1 unicode=1 agree\nodd.txt: lf=1 go=1 rust=1 universal=2 js=2 python=2 unicode=2 differ\n"
    );
    assert_eq!(files.err.borrow().as_str(), "");

    let stdin = FakeIo {
        files: Vec::new(),
        stdin: b"a\n".to_vec(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(cmd_agree::run(&["-".to_string()], &stdin), Ok(0));
    assert_eq!(
        stdin.out.borrow().as_str(),
        "<stdin>: lf=1 go=1 rust=1 universal=1 js=1 python=1 unicode=1 agree\n"
    );
    assert_eq!(stdin.err.borrow().as_str(), "");
}

#[test]
fn agree_read_failures_are_atomic_and_names_are_sanitized() {
    let missing = FakeIo {
        files: vec![("t.txt".to_string(), b"a\n".to_vec())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_agree::run(&["t.txt".to_string(), "missing.txt".to_string()], &missing),
        Err(EsError("missing.txt: cannot read the file".to_string()))
    );
    assert_eq!(missing.out.borrow().as_str(), "");
    assert_eq!(missing.err.borrow().as_str(), "");

    let sanitized = FakeIo {
        files: Vec::new(),
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_agree::run(&["café😀.txt".to_string()], &sanitized),
        Err(EsError("caf??.txt: cannot read the file".to_string()))
    );
    assert_eq!(sanitized.out.borrow().as_str(), "");
    assert_eq!(sanitized.err.borrow().as_str(), "");
}

#[test]
fn invalid_utf8_errors_are_atomic_for_both_commands() {
    let agree_io = FakeIo {
        files: vec![
            ("t.txt".to_string(), b"a\n".to_vec()),
            ("bad.txt".to_string(), vec![97, 255, 10]),
        ],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_agree::run(&["t.txt".to_string(), "bad.txt".to_string()], &agree_io),
        Err(EsError("bad.txt: input is not valid UTF-8".to_string()))
    );
    assert_eq!(agree_io.out.borrow().as_str(), "");
    assert_eq!(agree_io.err.borrow().as_str(), "");

    let lines_io = FakeIo {
        files: vec![("bad.txt".to_string(), vec![97, 255, 10])],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_lines::run(&["lf".to_string(), "bad.txt".to_string()], &lines_io),
        Err(EsError("bad.txt: input is not valid UTF-8".to_string()))
    );
    assert_eq!(lines_io.out.borrow().as_str(), "");
    assert_eq!(lines_io.err.borrow().as_str(), "");
}

#[test]
fn repeated_stdin_error_is_atomic() {
    let io = FakeIo {
        files: Vec::new(),
        stdin: b"a\n".to_vec(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cmd_agree::run(&["-".to_string(), "-".to_string()], &io),
        Err(EsError("standard input is named twice".to_string()))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}
