use std::collections::BTreeSet;

use eolscope::error::EsError;
use eolscope::io::{Inputs, Io, Source};
use eolscope::options::{Parsed, parse};
use eolscope::text::{plain, plural, show, table};

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
fn public_value_types_clone_compare_and_debug() {
    let error = EsError("one".to_string());
    assert_eq!(error.clone(), error);
    assert_ne!(error, EsError("two".to_string()));
    assert_eq!(format!("{error:?}"), "EsError(\"one\")");

    let parsed = Parsed {
        flags: BTreeSet::from(["--all".to_string()]),
        args: vec!["file".to_string()],
    };
    assert_eq!(parsed.clone(), parsed);
    assert_ne!(
        parsed,
        Parsed {
            flags: BTreeSet::new(),
            args: vec![]
        }
    );
    assert!(format!("{parsed:?}").contains("--all"));

    let source = Source {
        display: "file".to_string(),
        text: "a\r\nb".to_string(),
    };
    assert_eq!(source.clone(), source);
    assert_ne!(
        source,
        Source {
            display: "file".to_string(),
            text: "other".to_string()
        }
    );
    assert!(format!("{source:?}").contains("a\\r\\nb"));

    let io = FakeIo {
        files: vec![],
        stdin: vec![],
        out: Default::default(),
        err: Default::default(),
    };
    io.out("output");
    io.err("error");
    assert_eq!(io.out.borrow().as_str(), "output");
    assert_eq!(io.err.borrow().as_str(), "error");
}

#[test]
fn inputs_clone_equality_and_debug_use_backend_identity_and_state() {
    let io = FakeIo {
        files: vec![],
        stdin: b"x".to_vec(),
        out: Default::default(),
        err: Default::default(),
    };
    let other_io = FakeIo {
        files: vec![],
        stdin: b"x".to_vec(),
        out: Default::default(),
        err: Default::default(),
    };
    let inputs = Inputs::new(&io);
    let cloned = inputs.clone();
    assert_eq!(inputs, cloned);
    assert_ne!(inputs, Inputs::new(&other_io));
    assert_eq!(format!("{inputs:?}"), "Inputs { stdin_consumed: false }");

    let mut consumed = cloned;
    assert_eq!(
        consumed.read("-"),
        Ok(Source {
            display: "<stdin>".to_string(),
            text: "x".to_string()
        })
    );
    assert_ne!(inputs, consumed);
    assert_eq!(format!("{consumed:?}"), "Inputs { stdin_consumed: true }");
}

#[test]
fn plain_replaces_each_non_printable_or_non_ascii_scalar() {
    assert_eq!(plain(""), "");
    assert_eq!(plain("A \"~"), "A \"~");
    assert_eq!(plain("a\n\té😀\u{7f}"), "a?????");
    assert_eq!(plain("\u{feff}\r\u{85}\u{2028}\u{2029}"), "?????");
}

#[test]
fn show_adds_quotes_without_escaping_quotes() {
    assert_eq!(show(""), "\"\"");
    assert_eq!(show("a\"b"), "\"a\"b\"");
    assert_eq!(show("é😀\n"), "\"???\"");
    assert_eq!(show(" ~"), "\" ~\"");
}

#[test]
fn plural_uses_singular_only_for_one() {
    assert_eq!(plural(0, "file"), "0 files");
    assert_eq!(plural(1, "file"), "1 file");
    assert_eq!(plural(2, "file"), "2 files");
    assert_eq!(plural(1, "character"), "1 character");
    assert_eq!(plural(2, "problem"), "2 problems");
}

#[test]
fn table_sizes_columns_and_trims_final_padding() {
    assert_eq!(
        table(
            &["A", "LONG"],
            &[
                vec!["xx".to_string(), "z".to_string()],
                vec!["".to_string(), "tail".to_string()]
            ]
        ),
        vec!["A   LONG", "xx  z", "    tail"]
    );
    assert_eq!(table(&["TEXT"], &[]), vec!["TEXT"]);
    assert_eq!(table(&[], &[]), vec![""]);
}

#[test]
fn table_treats_missing_ragged_cells_as_empty() {
    assert_eq!(
        table(&["A", "B"], &[vec!["x".to_string()]]),
        vec!["A  B", "x"]
    );
}

#[test]
fn parse_keeps_arguments_and_deduplicates_sorted_flags() {
    assert_eq!(
        parse(&[], &[]),
        Ok(Parsed {
            flags: BTreeSet::new(),
            args: vec![]
        })
    );
    assert_eq!(
        parse(
            &[
                "x".to_string(),
                "--all".to_string(),
                "-".to_string(),
                "--all".to_string(),
                "-5".to_string()
            ],
            &["--all"]
        ),
        Ok(Parsed {
            flags: BTreeSet::from(["--all".to_string()]),
            args: vec!["x".to_string(), "-".to_string(), "-5".to_string()]
        })
    );
    assert_eq!(
        parse(&["--z".to_string(), "--a".to_string()], &["--z", "--a"]),
        Ok(Parsed {
            flags: BTreeSet::from(["--a".to_string(), "--z".to_string()]),
            args: vec![]
        })
    );
}

#[test]
fn parse_reports_the_first_unknown_option_exactly() {
    assert_eq!(
        parse(&["--oops".to_string(), "--bad".to_string()], &[]),
        Err(EsError("unknown option \"--oops\"".to_string()))
    );
    assert_eq!(
        parse(&["--".to_string()], &[]),
        Err(EsError("unknown option \"--\"".to_string()))
    );
    assert_eq!(
        parse(&["--help".to_string()], &[]),
        Err(EsError("unknown option \"--help\"".to_string()))
    );
    assert_eq!(
        parse(&["--all=true".to_string()], &["--all"]),
        Err(EsError("unknown option \"--all=true\"".to_string()))
    );
    assert_eq!(
        parse(&["--é😀".to_string()], &[]),
        Err(EsError("unknown option \"--??\"".to_string()))
    );
}

#[test]
fn inputs_preserve_unicode_paths_bom_cr_and_valid_stdin() {
    let io = FakeIo {
        files: vec![
            (
                "café😀.txt".to_string(),
                "\u{feff}a\r\nb\r😀".as_bytes().to_vec(),
            ),
            ("empty".to_string(), Vec::new()),
        ],
        stdin: b"s\r\n".to_vec(),
        out: Default::default(),
        err: Default::default(),
    };
    let mut inputs = Inputs::new(&io);
    assert_eq!(
        inputs.read("café😀.txt"),
        Ok(Source {
            display: "caf??.txt".to_string(),
            text: "\u{feff}a\r\nb\r😀".to_string()
        })
    );
    assert_eq!(
        inputs.read("café😀.txt"),
        Ok(Source {
            display: "caf??.txt".to_string(),
            text: "\u{feff}a\r\nb\r😀".to_string()
        })
    );
    assert_eq!(
        inputs.read("empty"),
        Ok(Source {
            display: "empty".to_string(),
            text: "".to_string()
        })
    );
    assert_eq!(
        inputs.read("-"),
        Ok(Source {
            display: "<stdin>".to_string(),
            text: "s\r\n".to_string()
        })
    );
    assert_eq!(
        inputs.read("-"),
        Err(EsError("standard input is named twice".to_string()))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn inputs_hide_backend_read_errors_for_unicode_paths() {
    let io = FakeIo {
        files: vec![],
        stdin: vec![],
        out: Default::default(),
        err: Default::default(),
    };
    let mut inputs = Inputs::new(&io);
    assert_eq!(
        inputs.read("café😀.txt"),
        Err(EsError("caf??.txt: cannot read the file".to_string()))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn inputs_reject_invalid_utf8_file_bytes_without_replacement() {
    let io = FakeIo {
        files: vec![("bad.txt".to_string(), vec![0x61, 0xFF, 0x0A])],
        stdin: vec![],
        out: Default::default(),
        err: Default::default(),
    };
    let mut inputs = Inputs::new(&io);
    assert_eq!(
        inputs.read("bad.txt"),
        Err(EsError("bad.txt: input is not valid UTF-8".to_string()))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn inputs_consume_stdin_on_invalid_utf8() {
    let io = FakeIo {
        files: vec![],
        stdin: vec![0x61, 0xFF, 0x0A],
        out: Default::default(),
        err: Default::default(),
    };
    let mut inputs = Inputs::new(&io);
    assert_eq!(
        inputs.read("-"),
        Err(EsError("<stdin>: input is not valid UTF-8".to_string()))
    );
    assert_eq!(
        inputs.read("-"),
        Err(EsError("standard input is named twice".to_string()))
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(io.err.borrow().as_str(), "");
}
