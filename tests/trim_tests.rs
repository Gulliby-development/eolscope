use eolscope::error::EsError;
use eolscope::{cmd_chars, cmd_trim};

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
fn usage_profile_and_option_errors_follow_precedence() {
    let io = FakeIo {
        files: vec![],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_trim::run(&[], &io),
        Err(EsError("usage: trim [--all] PROFILE FILE...".to_string()))
    );
    assert_eq!(
        cmd_trim::run(&["lf".to_string()], &io),
        Err(EsError("usage: trim [--all] PROFILE FILE...".to_string()))
    );
    assert_eq!(
        cmd_trim::run(&["LF".to_string(), "missing.txt".to_string()], &io),
        Err(EsError("unknown profile \"LF\"".to_string()))
    );
    assert_eq!(
        cmd_trim::run(&["--oops".to_string(), "LF".to_string()], &io),
        Err(EsError("unknown option \"--oops\"".to_string()))
    );
    assert_eq!(
        cmd_chars::run(&["85".to_string(), "--oops".to_string()], &io),
        Err(EsError("unknown option \"--oops\"".to_string()))
    );
    assert_eq!(cmd_trim::USAGE, "trim [--all] PROFILE FILE...");
    assert_eq!(cmd_chars::USAGE, "chars [CODE...]");
    assert!(io.out.borrow().is_empty());
    assert!(io.err.borrow().is_empty());
}

#[test]
fn trim_reports_the_exact_disagreement_rows() {
    let io = FakeIo {
        files: vec![(
            "t.txt".to_string(),
            "a \nb\u{00a0}\nc\u{feff}\nd\u{3000}\ne\t\u{00a0}\nf\u{0085}\ng\n"
                .as_bytes()
                .to_vec(),
        )],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_trim::run(&["lf".to_string(), "t.txt".to_string()], &io),
        Ok(1)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        "FILE   LINE  js-trim  py-strip  rs-trim  java-trim  java-strip  VERDICT\n\
         t.txt  2     1        1         1        0          0           differ\n\
         t.txt  3     1        0         0        0          0           differ\n\
         t.txt  4     1        1         1        0          1           differ\n\
         t.txt  5     2        2         2        0          0           differ\n\
         t.txt  6     0        1         1        0          0           differ\n"
    );
    assert!(io.err.borrow().is_empty());
}

#[test]
fn trim_all_lists_every_row_and_accepts_repeated_option() {
    let io = FakeIo {
        files: vec![(
            "t.txt".to_string(),
            "a \nb\u{00a0}\nc\n".as_bytes().to_vec(),
        )],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_trim::run(
            &[
                "--all".to_string(),
                "lf".to_string(),
                "--all".to_string(),
                "t.txt".to_string(),
            ],
            &io
        ),
        Ok(1)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        "FILE   LINE  js-trim  py-strip  rs-trim  java-trim  java-strip  VERDICT\n\
         t.txt  1     1        1         1        1          1           same\n\
         t.txt  2     1        1         1        0          0           differ\n\
         t.txt  3     0        0         0        0          0           same\n"
    );
}

#[test]
fn trim_handles_empty_same_astral_and_repeated_files() {
    let io = FakeIo {
        files: vec![
            ("same.txt".to_string(), "😀 \n".as_bytes().to_vec()),
            ("empty.txt".to_string(), Vec::new()),
            ("repeat.txt".to_string(), "x\u{00a0}\n".as_bytes().to_vec()),
        ],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_trim::run(&["lf".to_string(), "same.txt".to_string()], &io),
        Ok(0)
    );
    assert_eq!(io.out.borrow().as_str(), "none\n");
    io.out.borrow_mut().clear();
    assert_eq!(
        cmd_trim::run(
            &[
                "--all".to_string(),
                "lf".to_string(),
                "empty.txt".to_string()
            ],
            &io
        ),
        Ok(0)
    );
    assert_eq!(io.out.borrow().as_str(), "none\n");
    io.out.borrow_mut().clear();
    assert_eq!(
        cmd_trim::run(
            &[
                "--all".to_string(),
                "lf".to_string(),
                "same.txt".to_string(),
                "repeat.txt".to_string(),
                "same.txt".to_string(),
            ],
            &io
        ),
        Ok(1)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        r#"FILE        LINE  js-trim  py-strip  rs-trim  java-trim  java-strip  VERDICT
same.txt    1     1        1         1        1          1           same
repeat.txt  1     1        1         1        0          0           differ
same.txt    1     1        1         1        1          1           same
"#
    );
    assert!(io.err.borrow().is_empty());
}

#[test]
fn trim_uses_profile_specific_final_cr_content() {
    let io = FakeIo {
        files: vec![("t.txt".to_string(), b"a\r".to_vec())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_trim::run(
            &["--all".to_string(), "lf".to_string(), "t.txt".to_string()],
            &io
        ),
        Ok(0)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        "FILE   LINE  js-trim  py-strip  rs-trim  java-trim  java-strip  VERDICT\n\
         t.txt  1     1        1         1        1          1           same\n"
    );
    io.out.borrow_mut().clear();
    assert_eq!(
        cmd_trim::run(
            &["--all".to_string(), "rust".to_string(), "t.txt".to_string()],
            &io
        ),
        Ok(0)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        "FILE   LINE  js-trim  py-strip  rs-trim  java-trim  java-strip  VERDICT\n\
         t.txt  1     1        1         1        1          1           same\n"
    );
    io.out.borrow_mut().clear();
    assert_eq!(
        cmd_trim::run(
            &["--all".to_string(), "go".to_string(), "t.txt".to_string()],
            &io
        ),
        Ok(0)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        "FILE   LINE  js-trim  py-strip  rs-trim  java-trim  java-strip  VERDICT\n\
         t.txt  1     0        0         0        0          0           same\n"
    );
    io.out.borrow_mut().clear();
    assert_eq!(
        cmd_trim::run(
            &[
                "--all".to_string(),
                "universal".to_string(),
                "t.txt".to_string()
            ],
            &io
        ),
        Ok(0)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        "FILE   LINE  js-trim  py-strip  rs-trim  java-trim  java-strip  VERDICT\n\
         t.txt  1     0        0         0        0          0           same\n"
    );
}

#[test]
fn chars_rows_match_literal_scalar_membership_cells() {
    assert_eq!(
        cmd_chars::row(13),
        vec![
            "U+000D", "CR", ".", ".", ".", "T", "T", "T", "T", "x", "x", "x", "x", "x"
        ]
    );
    assert_eq!(
        cmd_chars::row(28),
        vec![
            "U+001C", "FS", ".", ".", ".", ".", ".", "T", ".", ".", "x", ".", "x", "x"
        ]
    );
    assert_eq!(
        cmd_chars::row(133),
        vec![
            "U+0085", "NEL", ".", ".", ".", ".", ".", "T", "T", ".", "x", "x", ".", "."
        ]
    );
    assert_eq!(
        cmd_chars::row(65279),
        vec![
            "U+FEFF", "BOM", ".", ".", ".", ".", ".", ".", ".", "x", ".", ".", ".", "."
        ]
    );
    assert_eq!(
        cmd_chars::row(128512),
        vec![
            "U+1F600", "-", ".", ".", ".", ".", ".", ".", ".", ".", ".", ".", ".", "."
        ]
    );
}

#[test]
fn chars_specials_and_default_table_are_exact() {
    assert_eq!(
        cmd_chars::SPECIALS,
        [
            (0x0009, "TAB"),
            (0x000A, "LF"),
            (0x000B, "VT"),
            (0x000C, "FF"),
            (0x000D, "CR"),
            (0x001C, "FS"),
            (0x001D, "GS"),
            (0x001E, "RS"),
            (0x001F, "US"),
            (0x0020, "SPACE"),
            (0x0085, "NEL"),
            (0x00A0, "NBSP"),
            (0x2007, "FIGURESP"),
            (0x2028, "LS"),
            (0x2029, "PS"),
            (0x202F, "NNBSP"),
            (0x3000, "IDEOSP"),
            (0xFEFF, "BOM"),
            (0x200B, "ZWSP"),
        ]
    );
    let io = FakeIo {
        files: vec![],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(cmd_chars::run(&[], &io), Ok(0));
    assert_eq!(
        io.out.borrow().as_str(),
        "CODE    NAME      lf  go  rust  universal  js  python  unicode  js-trim  py-strip  rs-trim  java-trim  java-strip\n\
         U+0009  TAB       .   .   .     .          .   .       .        x        x         x        x          x\n\
         U+000A  LF        T   T   T     T          T   T       T        x        x         x        x          x\n\
         U+000B  VT        .   .   .     .          .   T       T        x        x         x        x          x\n\
         U+000C  FF        .   .   .     .          .   T       T        x        x         x        x          x\n\
         U+000D  CR        .   .   .     T          T   T       T        x        x         x        x          x\n\
         U+001C  FS        .   .   .     .          .   T       .        .        x         .        x          x\n\
         U+001D  GS        .   .   .     .          .   T       .        .        x         .        x          x\n\
         U+001E  RS        .   .   .     .          .   T       .        .        x         .        x          x\n\
         U+001F  US        .   .   .     .          .   .       .        .        x         .        x          x\n\
         U+0020  SPACE     .   .   .     .          .   .       .        x        x         x        x          x\n\
         U+0085  NEL       .   .   .     .          .   T       T        .        x         x        .          .\n\
         U+00A0  NBSP      .   .   .     .          .   .       .        x        x         x        .          .\n\
         U+2007  FIGURESP  .   .   .     .          .   .       .        x        x         x        .          .\n\
         U+2028  LS        .   .   .     .          T   T       T        x        x         x        .          x\n\
         U+2029  PS        .   .   .     .          T   T       T        x        x         x        .          x\n\
         U+202F  NNBSP     .   .   .     .          .   .       .        x        x         x        .          .\n\
         U+3000  IDEOSP    .   .   .     .          .   .       .        x        x         x        .          x\n\
         U+FEFF  BOM       .   .   .     .          .   .       .        x        .         .        .          .\n\
         U+200B  ZWSP      .   .   .     .          .   .       .        .        .         .        .          .\n"
    );
    assert!(io.err.borrow().is_empty());
}

#[test]
fn chars_prints_the_exact_four_code_table() {
    let io = FakeIo {
        files: vec![],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_chars::run(
            &[
                "U+0085".to_string(),
                "2029".to_string(),
                "0041".to_string(),
                "u+1F600".to_string(),
            ],
            &io
        ),
        Ok(0)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        r#"CODE     NAME  lf  go  rust  universal  js  python  unicode  js-trim  py-strip  rs-trim  java-trim  java-strip
U+0085   NEL   .   .   .     .          .   T       T        .        x         x        .          .
U+2029   PS    .   .   .     .          T   T       T        x        x         x        .          x
U+0041   -     .   .   .     .          .   .       .        .        .         .        .          .
U+1F600  -     .   .   .     .          .   .       .        .        .         .        .          .
"#
    );
    assert!(io.err.borrow().is_empty());
}

#[test]
fn chars_accepts_boundaries_normalizes_and_preserves_duplicates() {
    let io = FakeIo {
        files: vec![],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_chars::run(
            &[
                "0000".to_string(),
                "D7FF".to_string(),
                "E000".to_string(),
                "10FFFF".to_string(),
                "000041".to_string(),
                "00a0".to_string(),
            ],
            &io
        ),
        Ok(0)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        r#"CODE      NAME  lf  go  rust  universal  js  python  unicode  js-trim  py-strip  rs-trim  java-trim  java-strip
U+0000    -     .   .   .     .          .   .       .        .        .         .        x          .
U+D7FF    -     .   .   .     .          .   .       .        .        .         .        .          .
U+E000    -     .   .   .     .          .   .       .        .        .         .        .          .
U+10FFFF  -     .   .   .     .          .   .       .        .        .         .        .          .
U+0041    -     .   .   .     .          .   .       .        .        .         .        .          .
U+00A0    NBSP  .   .   .     .          .   .       .        x        x         x        .          .
"#
    );
    io.out.borrow_mut().clear();
    assert_eq!(
        cmd_chars::run(&["0041".to_string(), "u+0041".to_string()], &io),
        Ok(0)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        "CODE    NAME  lf  go  rust  universal  js  python  unicode  js-trim  py-strip  rs-trim  java-trim  java-strip\n\
         U+0041  -     .   .   .     .          .   .       .        .        .         .        .          .\n\
         U+0041  -     .   .   .     .          .   .       .        .        .         .        .          .\n"
    );
}

#[test]
fn chars_rejects_invalid_syntax_forms() {
    let io = FakeIo {
        files: vec![],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_chars::run(&["85".to_string()], &io),
        Err(EsError("invalid code \"85\"".to_string()))
    );
    assert_eq!(
        cmd_chars::run(&["0000041".to_string()], &io),
        Err(EsError("invalid code \"0000041\"".to_string()))
    );
    assert_eq!(
        cmd_chars::run(&["0x0041".to_string()], &io),
        Err(EsError("invalid code \"0x0041\"".to_string()))
    );
    assert_eq!(
        cmd_chars::run(&["+0041".to_string()], &io),
        Err(EsError("invalid code \"+0041\"".to_string()))
    );
    assert_eq!(
        cmd_chars::run(&[" 0041".to_string()], &io),
        Err(EsError("invalid code \" 0041\"".to_string()))
    );
    assert_eq!(
        cmd_chars::run(&["0041 ".to_string()], &io),
        Err(EsError("invalid code \"0041 \"".to_string()))
    );
    assert_eq!(
        cmd_chars::run(&["".to_string()], &io),
        Err(EsError("invalid code \"\"".to_string()))
    );
    assert!(io.out.borrow().is_empty());
    assert!(io.err.borrow().is_empty());
}

#[test]
fn chars_rejects_non_scalars_and_reports_the_first_invalid_code() {
    let io = FakeIo {
        files: vec![],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_chars::run(&["U+D800".to_string()], &io),
        Err(EsError("invalid code \"U+D800\"".to_string()))
    );
    assert_eq!(
        cmd_chars::run(&["DFFF".to_string()], &io),
        Err(EsError("invalid code \"DFFF\"".to_string()))
    );
    assert_eq!(
        cmd_chars::run(&["110000".to_string()], &io),
        Err(EsError("invalid code \"110000\"".to_string()))
    );
    assert_eq!(
        cmd_chars::run(&["U+GGGG".to_string()], &io),
        Err(EsError("invalid code \"U+GGGG\"".to_string()))
    );
    assert_eq!(
        cmd_chars::run(&["U+".to_string()], &io),
        Err(EsError("invalid code \"U+\"".to_string()))
    );
    assert_eq!(
        cmd_chars::run(&["U+D800".to_string(), "85".to_string()], &io),
        Err(EsError("invalid code \"U+D800\"".to_string()))
    );
    assert!(io.out.borrow().is_empty());
    assert!(io.err.borrow().is_empty());
}

#[test]
fn trim_errors_after_prior_reads_are_atomic() {
    let io = FakeIo {
        files: vec![
            ("valid.txt".to_string(), b"a \n".to_vec()),
            ("bad.txt".to_string(), vec![97, 255, 10]),
        ],
        stdin: b"a\n".to_vec(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_trim::run(
            &[
                "lf".to_string(),
                "valid.txt".to_string(),
                "missing.txt".to_string(),
            ],
            &io
        ),
        Err(EsError("missing.txt: cannot read the file".to_string()))
    );
    assert!(io.out.borrow().is_empty());
    assert!(io.err.borrow().is_empty());
    assert_eq!(
        cmd_trim::run(
            &[
                "lf".to_string(),
                "valid.txt".to_string(),
                "bad.txt".to_string()
            ],
            &io
        ),
        Err(EsError("bad.txt: input is not valid UTF-8".to_string()))
    );
    assert!(io.out.borrow().is_empty());
    assert!(io.err.borrow().is_empty());
    assert_eq!(
        cmd_trim::run(&["lf".to_string(), "-".to_string(), "-".to_string()], &io),
        Err(EsError("standard input is named twice".to_string()))
    );
    assert!(io.out.borrow().is_empty());
    assert!(io.err.borrow().is_empty());
}

#[test]
fn invalid_later_code_and_sanitized_display_leave_streams_empty() {
    let io = FakeIo {
        files: vec![],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };
    assert_eq!(
        cmd_chars::run(&["0041".to_string(), "".to_string()], &io),
        Err(EsError("invalid code \"\"".to_string()))
    );
    assert!(io.out.borrow().is_empty());
    assert!(io.err.borrow().is_empty());
    assert_eq!(
        cmd_trim::run(&["lf".to_string(), "café😀.txt".to_string()], &io),
        Err(EsError("caf??.txt: cannot read the file".to_string()))
    );
    assert!(io.out.borrow().is_empty());
    assert!(io.err.borrow().is_empty());
}
