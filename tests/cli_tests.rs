use eolscope::cli::{self, USAGE};
use std::io::Write;
use std::process::{Command, Stdio};

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
fn help_is_exact_and_written_without_error() {
    assert_eq!(
        USAGE,
        "usage: eolscope lines PROFILE FILE\nusage: eolscope agree FILE...\nusage: eolscope kinds FILE...\nusage: eolscope where OFFSET FILE\nusage: eolscope trim [--all] PROFILE FILE...\nusage: eolscope chars [CODE...]\nusage: eolscope convert [--final-newline] [--strip-bom] FROM TO FILE\nusage: eolscope check POLICY FILE...\n"
    );
    let io = FakeIo {
        files: Vec::new(),
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(cli::run(&["--help".to_string()], &io), 0);
    assert_eq!(io.out.borrow().as_str(), USAGE);
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn command_selection_errors_use_the_cli_envelope() {
    let io = FakeIo {
        files: Vec::new(),
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(cli::run(&[], &io), 2);
    assert_eq!(cli::run(&["frob".to_string(), "x".to_string()], &io), 2);
    assert_eq!(
        cli::run(&["--help".to_string(), "lines".to_string()], &io),
        2
    );
    assert_eq!(cli::run(&["café😀".to_string()], &io), 2);
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(
        io.err.borrow().as_str(),
        "eolscope: missing command (lines, agree, kinds, where, trim, chars, convert or check)\neolscope: unknown command \"frob\"\neolscope: unknown command \"--help\"\neolscope: unknown command \"caf??\"\n"
    );
}

#[test]
fn lines_route_formats_an_empty_file() {
    let io = FakeIo {
        files: vec![("t.txt".to_string(), Vec::new())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cli::run(
            &["lines".to_string(), "lf".to_string(), "t.txt".to_string()],
            &io
        ),
        0
    );
    assert_eq!(io.out.borrow().as_str(), "none\n");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn agree_route_covers_empty_and_differing_files() {
    let empty = FakeIo {
        files: vec![("t.txt".to_string(), Vec::new())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cli::run(&["agree".to_string(), "t.txt".to_string()], &empty),
        0
    );
    assert_eq!(
        empty.out.borrow().as_str(),
        "t.txt: lf=0 go=0 rust=0 universal=0 js=0 python=0 unicode=0 agree\n"
    );
    assert_eq!(empty.err.borrow().as_str(), "");

    let differing = FakeIo {
        files: vec![("t.txt".to_string(), b"a\rb".to_vec())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cli::run(&["agree".to_string(), "t.txt".to_string()], &differing),
        1
    );
    assert_eq!(
        differing.out.borrow().as_str(),
        "t.txt: lf=1 go=1 rust=1 universal=2 js=2 python=2 unicode=2 differ\n"
    );
    assert_eq!(differing.err.borrow().as_str(), "");
}

#[test]
fn kinds_route_covers_empty_and_mixed_files() {
    let empty = FakeIo {
        files: vec![("t.txt".to_string(), Vec::new())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cli::run(&["kinds".to_string(), "t.txt".to_string()], &empty),
        0
    );
    assert_eq!(
        empty.out.borrow().as_str(),
        "t.txt:\n  lines  0\n  style  none\n  final  -\n  bom    no\n"
    );
    assert_eq!(empty.err.borrow().as_str(), "");

    let mixed = FakeIo {
        files: vec![("t.txt".to_string(), b"a\nb\r".to_vec())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cli::run(&["kinds".to_string(), "t.txt".to_string()], &mixed),
        1
    );
    assert_eq!(
        mixed.out.borrow().as_str(),
        "t.txt:\n  lines  2\n  LF     1\n  CR     1\n  style  mixed\n  final  yes\n  bom    no\n"
    );
    assert_eq!(mixed.err.borrow().as_str(), "");
}

#[test]
fn where_route_formats_empty_eof_for_each_profile() {
    let io = FakeIo {
        files: vec![("t.txt".to_string(), Vec::new())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cli::run(
            &["where".to_string(), "0".to_string(), "t.txt".to_string()],
            &io
        ),
        0
    );
    assert_eq!(
        io.out.borrow().as_str(),
        "PROFILE    LINE  COL  AT\nlf         1     1    end\ngo         1     1    end\nrust       1     1    end\nuniversal  1     1    end\njs         1     1    end\npython     1     1    end\nunicode    1     1    end\n"
    );
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn trim_route_covers_empty_and_disagreement() {
    let empty = FakeIo {
        files: vec![("t.txt".to_string(), Vec::new())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cli::run(
            &["trim".to_string(), "lf".to_string(), "t.txt".to_string()],
            &empty
        ),
        0
    );
    assert_eq!(empty.out.borrow().as_str(), "none\n");
    assert_eq!(empty.err.borrow().as_str(), "");

    let differing = FakeIo {
        files: vec![("t.txt".to_string(), "b\u{00a0}\n".as_bytes().to_vec())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cli::run(
            &["trim".to_string(), "lf".to_string(), "t.txt".to_string()],
            &differing
        ),
        1
    );
    assert_eq!(
        differing.out.borrow().as_str(),
        "FILE   LINE  js-trim  py-strip  rs-trim  java-trim  java-strip  VERDICT\nt.txt  1     1        1         1        0          0           differ\n"
    );
    assert_eq!(differing.err.borrow().as_str(), "");
}

#[test]
fn chars_route_formats_one_literal_scalar() {
    let io = FakeIo {
        files: Vec::new(),
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(cli::run(&["chars".to_string(), "0041".to_string()], &io), 0);
    assert_eq!(
        io.out.borrow().as_str(),
        "CODE    NAME  lf  go  rust  universal  js  python  unicode  js-trim  py-strip  rs-trim  java-trim  java-strip\nU+0041  -     .   .   .     .          .   .       .        .        .         .        .          .\n"
    );
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn convert_route_preserves_empty_and_forwards_flags() {
    let empty = FakeIo {
        files: vec![("t.txt".to_string(), Vec::new())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cli::run(
            &[
                "convert".to_string(),
                "lf".to_string(),
                "crlf".to_string(),
                "t.txt".to_string()
            ],
            &empty
        ),
        0
    );
    assert_eq!(empty.out.borrow().as_str(), "");
    assert_eq!(empty.err.borrow().as_str(), "");

    let flagged = FakeIo {
        files: vec![("t.txt".to_string(), "\u{feff}a".as_bytes().to_vec())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cli::run(
            &[
                "convert".to_string(),
                "--final-newline".to_string(),
                "--strip-bom".to_string(),
                "lf".to_string(),
                "crlf".to_string(),
                "t.txt".to_string()
            ],
            &flagged
        ),
        0
    );
    assert_eq!(flagged.out.borrow().as_str(), "a\r\n");
    assert_eq!(flagged.err.borrow().as_str(), "");
}

#[test]
fn check_route_covers_clean_and_negative_files() {
    let empty = FakeIo {
        files: vec![("t.txt".to_string(), Vec::new())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cli::run(
            &["check".to_string(), "lf".to_string(), "t.txt".to_string()],
            &empty
        ),
        0
    );
    assert_eq!(empty.out.borrow().as_str(), "ok\n");
    assert_eq!(empty.err.borrow().as_str(), "");

    let missing_final = FakeIo {
        files: vec![("t.txt".to_string(), b"a".to_vec())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cli::run(
            &[
                "check".to_string(),
                "final".to_string(),
                "t.txt".to_string()
            ],
            &missing_final
        ),
        1
    );
    assert_eq!(
        missing_final.out.borrow().as_str(),
        "t.txt:1:2: EV002 missing final line terminator\n1 problem\n"
    );
    assert_eq!(missing_final.err.borrow().as_str(), "");
}

#[test]
fn validation_errors_follow_the_cli_envelope() {
    let io = FakeIo {
        files: vec![("t.txt".to_string(), b"ab".to_vec())],
        stdin: Vec::new(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cli::run(
            &[
                "lines".to_string(),
                "--oops".to_string(),
                "LF".to_string(),
                "missing.txt".to_string()
            ],
            &io
        ),
        2
    );
    assert_eq!(cli::run(&["agree".to_string()], &io), 2);
    assert_eq!(cli::run(&["trim".to_string(), "lf".to_string()], &io), 2);
    assert_eq!(
        cli::run(
            &[
                "lines".to_string(),
                "LF".to_string(),
                "missing.txt".to_string()
            ],
            &io
        ),
        2
    );
    assert_eq!(
        cli::run(
            &[
                "convert".to_string(),
                "lf".to_string(),
                "tab".to_string(),
                "missing.txt".to_string()
            ],
            &io
        ),
        2
    );
    assert_eq!(
        cli::run(
            &[
                "where".to_string(),
                "007".to_string(),
                "missing.txt".to_string()
            ],
            &io
        ),
        2
    );
    assert_eq!(
        cli::run(
            &["where".to_string(), "9".to_string(), "t.txt".to_string()],
            &io
        ),
        2
    );
    assert_eq!(
        cli::run(
            &[
                "check".to_string(),
                "final,nofinal,x".to_string(),
                "missing.txt".to_string()
            ],
            &io
        ),
        2
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(
        io.err.borrow().as_str(),
        "eolscope: unknown option \"--oops\"\neolscope: usage: agree FILE...\neolscope: usage: trim [--all] PROFILE FILE...\neolscope: unknown profile \"LF\"\neolscope: unknown style \"tab\"\neolscope: invalid offset \"007\"\neolscope: offset 9 is beyond the end (2 characters)\neolscope: conflicting items final and nofinal\n"
    );
}

#[test]
fn read_errors_are_atomic_and_streams_stay_empty() {
    let io = FakeIo {
        files: vec![
            ("t.txt".to_string(), b"a\n".to_vec()),
            ("bad.txt".to_string(), vec![0x61, 0xFF, 0x0A]),
        ],
        stdin: b"a\n".to_vec(),
        out: Default::default(),
        err: Default::default(),
    };

    assert_eq!(
        cli::run(
            &[
                "agree".to_string(),
                "t.txt".to_string(),
                "missing.txt".to_string()
            ],
            &io
        ),
        2
    );
    assert_eq!(
        cli::run(
            &["kinds".to_string(), "-".to_string(), "-".to_string()],
            &io
        ),
        2
    );
    assert_eq!(
        cli::run(&["kinds".to_string(), "café😀.txt".to_string()], &io),
        2
    );
    assert_eq!(
        cli::run(
            &["lines".to_string(), "lf".to_string(), "bad.txt".to_string()],
            &io
        ),
        2
    );
    assert_eq!(
        cli::run(
            &["chars".to_string(), "0041".to_string(), "".to_string()],
            &io
        ),
        2
    );
    assert_eq!(
        cli::run(
            &[
                "trim".to_string(),
                "lf".to_string(),
                "t.txt".to_string(),
                "missing.txt".to_string()
            ],
            &io
        ),
        2
    );
    assert_eq!(
        cli::run(
            &[
                "check".to_string(),
                "lf".to_string(),
                "t.txt".to_string(),
                "missing.txt".to_string()
            ],
            &io
        ),
        2
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(
        io.err.borrow().as_str(),
        "eolscope: missing.txt: cannot read the file\neolscope: standard input is named twice\neolscope: caf??.txt: cannot read the file\neolscope: bad.txt: input is not valid UTF-8\neolscope: invalid code \"\"\neolscope: missing.txt: cannot read the file\neolscope: missing.txt: cannot read the file\n"
    );
}

#[test]
fn malformed_stdin_utf8_is_rejected_by_the_process() {
    let spawn_result = Command::new(env!("CARGO_BIN_EXE_eolscope"))
        .args(["lines", "lf", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    assert!(spawn_result.is_ok());
    let mut child = match spawn_result {
        Ok(child) => child,
        Err(_) => return,
    };

    let stdin = child.stdin.take();
    assert!(stdin.is_some());
    let mut stdin = match stdin {
        Some(stdin) => stdin,
        None => return,
    };
    assert!(stdin.write_all(&[0x61, 0xFF, 0x0A]).is_ok());
    drop(stdin);

    let output_result = child.wait_with_output();
    assert!(output_result.is_ok());
    let output = match output_result {
        Ok(output) => output,
        Err(_) => return,
    };
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        b"eolscope: <stdin>: input is not valid UTF-8\n"
    );
}
