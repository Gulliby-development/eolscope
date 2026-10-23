use eolscope::cmd_check;
use eolscope::error::EsError;

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

fn fake_io(files: Vec<(String, Vec<u8>)>, stdin: Vec<u8>) -> FakeIo {
    FakeIo {
        files,
        stdin,
        out: Default::default(),
        err: Default::default(),
    }
}

#[test]
fn usage_is_exact_and_missing_arguments_are_rejected() {
    assert_eq!(cmd_check::USAGE, "check POLICY FILE...");

    let io = fake_io(vec![], Vec::new());
    assert_eq!(
        cmd_check::run(&[], &io),
        Err(EsError("usage: check POLICY FILE...".to_string()))
    );
    assert_eq!(
        cmd_check::run(&["lf".to_string()], &io),
        Err(EsError("usage: check POLICY FILE...".to_string()))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn clean_and_empty_files_produce_ok() {
    let clean_io = fake_io(vec![("clean.txt".to_string(), b"x\n".to_vec())], Vec::new());
    assert_eq!(
        cmd_check::run(
            &["lf,final".to_string(), "clean.txt".to_string()],
            &clean_io
        ),
        Ok(0)
    );
    assert_eq!(clean_io.out.borrow().as_str(), "ok\n");
    assert_eq!(clean_io.err.borrow().as_str(), "");

    let empty_io = fake_io(vec![("empty.txt".to_string(), Vec::new())], Vec::new());
    assert_eq!(
        cmd_check::run(
            &[
                "lf,final,nobom,notrail".to_string(),
                "empty.txt".to_string()
            ],
            &empty_io
        ),
        Ok(0)
    );
    assert_eq!(empty_io.out.borrow().as_str(), "ok\n");
    assert_eq!(empty_io.err.borrow().as_str(), "");
}

#[test]
fn one_problem_uses_singular_summary_and_explicit_lf() {
    let io = fake_io(vec![("t.txt".to_string(), b"a\nb".to_vec())], Vec::new());
    assert_eq!(
        cmd_check::run(&["lf,final".to_string(), "t.txt".to_string()], &io),
        Ok(1)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        "t.txt:2:2: EV002 missing final line terminator\n1 problem\n"
    );
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn repeated_file_arguments_repeat_diagnostics_and_count_them_all() {
    let io = fake_io(
        vec![
            ("t.txt".to_string(), b"a\nb".to_vec()),
            ("clean.txt".to_string(), b"x\n".to_vec()),
        ],
        Vec::new(),
    );
    assert_eq!(
        cmd_check::run(
            &[
                "lf,final".to_string(),
                "t.txt".to_string(),
                "clean.txt".to_string(),
                "t.txt".to_string(),
            ],
            &io
        ),
        Ok(1)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        "t.txt:2:2: EV002 missing final line terminator\nt.txt:2:2: EV002 missing final line terminator\n2 problems\n"
    );
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn stdin_and_sanitized_unicode_filename_keep_argument_order() {
    let io = fake_io(
        vec![("café😀.txt".to_string(), b"b".to_vec())],
        b"a".to_vec(),
    );
    assert_eq!(
        cmd_check::run(
            &[
                "final".to_string(),
                "-".to_string(),
                "café😀.txt".to_string(),
            ],
            &io
        ),
        Ok(1)
    );
    assert_eq!(
        io.out.borrow().as_str(),
        "<stdin>:1:2: EV002 missing final line terminator\ncaf??.txt:1:2: EV002 missing final line terminator\n2 problems\n"
    );
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn later_missing_file_discards_prior_diagnostics() {
    let io = fake_io(vec![("t.txt".to_string(), b"a".to_vec())], Vec::new());
    assert_eq!(
        cmd_check::run(
            &[
                "final".to_string(),
                "t.txt".to_string(),
                "missing.txt".to_string(),
            ],
            &io
        ),
        Err(EsError("missing.txt: cannot read the file".to_string()))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn later_invalid_utf8_file_discards_prior_diagnostics() {
    let io = fake_io(
        vec![
            ("t.txt".to_string(), b"a".to_vec()),
            ("bad.txt".to_string(), vec![97, 255, 10]),
        ],
        Vec::new(),
    );
    assert_eq!(
        cmd_check::run(
            &[
                "final".to_string(),
                "t.txt".to_string(),
                "bad.txt".to_string(),
            ],
            &io
        ),
        Err(EsError("bad.txt: input is not valid UTF-8".to_string()))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn duplicate_stdin_is_an_atomic_error_after_first_input() {
    let io = fake_io(vec![], b"a".to_vec());
    assert_eq!(
        cmd_check::run(
            &["final".to_string(), "-".to_string(), "-".to_string()],
            &io
        ),
        Err(EsError("standard input is named twice".to_string()))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn options_are_validated_before_arity_policy_or_file_reads() {
    let rows = [
        (vec!["--oops".to_string()], "unknown option \"--oops\""),
        (
            vec![
                "lf".to_string(),
                "missing.txt".to_string(),
                "--help".to_string(),
            ],
            "unknown option \"--help\"",
        ),
    ];
    for (args, expected) in rows {
        let io = fake_io(vec![], Vec::new());
        assert_eq!(
            cmd_check::run(&args, &io),
            Err(EsError(expected.to_string()))
        );
        assert_eq!(io.out.borrow().as_str(), "");
        assert_eq!(io.err.borrow().as_str(), "");
    }
}

#[test]
fn empty_policy_and_empty_items_are_reported_before_reading_files() {
    let rows = [
        (String::new(), "empty policy"),
        ("lf,,final".to_string(), "empty item 2"),
    ];
    for (policy, expected) in rows {
        let io = fake_io(vec![], Vec::new());
        assert_eq!(
            cmd_check::run(&[policy, "missing.txt".to_string()], &io),
            Err(EsError(expected.to_string()))
        );
        assert_eq!(io.out.borrow().as_str(), "");
        assert_eq!(io.err.borrow().as_str(), "");
    }
}

#[test]
fn style_conflicts_and_unknown_items_preserve_policy_errors() {
    let rows = [
        ("lf,crlf", "two line styles lf and crlf"),
        ("lf,CRLF", "unknown item 2 \"CRLF\""),
    ];
    for (policy, expected) in rows {
        let io = fake_io(vec![], Vec::new());
        assert_eq!(
            cmd_check::run(&[policy.to_string(), "missing.txt".to_string()], &io),
            Err(EsError(expected.to_string()))
        );
        assert_eq!(io.out.borrow().as_str(), "");
        assert_eq!(io.err.borrow().as_str(), "");
    }
}

#[test]
fn final_conflicts_and_repeat_idempotence_are_preserved() {
    let conflict_io = fake_io(vec![], Vec::new());
    assert_eq!(
        cmd_check::run(
            &["final,nofinal,x".to_string(), "missing.txt".to_string(),],
            &conflict_io
        ),
        Err(EsError("conflicting items final and nofinal".to_string()))
    );
    assert_eq!(conflict_io.out.borrow().as_str(), "");
    assert_eq!(conflict_io.err.borrow().as_str(), "");

    let repeat_io = fake_io(vec![("t.txt".to_string(), b"a\n".to_vec())], Vec::new());
    assert_eq!(
        cmd_check::run(
            &["lf,lf,final,final".to_string(), "t.txt".to_string()],
            &repeat_io
        ),
        Ok(0)
    );
    assert_eq!(repeat_io.out.borrow().as_str(), "ok\n");
    assert_eq!(repeat_io.err.borrow().as_str(), "");
}
