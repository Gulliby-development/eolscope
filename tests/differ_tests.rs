use std::cell::RefCell;

use eolscope::cli;
use eolscope::io::Io;

struct FakeIo {
    out: RefCell<String>,
    err: RefCell<String>,
}

impl Io for FakeIo {
    fn read_stdin(&self) -> Result<Vec<u8>, String> {
        Ok(b"a\rb".to_vec())
    }

    fn read_file(&self, path: &str) -> Result<Vec<u8>, String> {
        match path {
            "same.txt" => Ok(b"a\nb\n".to_vec()),
            "cr.txt" => Ok(b"a\rb".to_vec()),
            "empty.txt" => Ok(Vec::new()),
            "ls.txt" => Ok("a\u{2028}b\n".as_bytes().to_vec()),
            _ => Err("no such file".to_string()),
        }
    }

    fn out(&self, text: &str) {
        self.out.borrow_mut().push_str(text);
    }

    fn err(&self, text: &str) {
        self.err.borrow_mut().push_str(text);
    }
}

const SAME: &str = "same.txt: lf=2 go=2 rust=2 universal=2 js=2 python=2 unicode=2 agree\n";
const CR: &str = "cr.txt: lf=1 go=1 rust=1 universal=2 js=2 python=2 unicode=2 differ\n";
const LS: &str = "ls.txt: lf=1 go=1 rust=1 universal=1 js=2 python=2 unicode=2 differ\n";
const STDIN: &str = "<stdin>: lf=1 go=1 rust=1 universal=2 js=2 python=2 unicode=2 differ\n";

// (command line, exit status, standard output, standard error)
const ROWS: &[(&[&str], i32, &str, &str)] = &[
    (&["agree", "--differ", "same.txt"], 0, "", ""),
    (&["agree", "--differ", "cr.txt"], 1, CR, ""),
    (&["agree", "--differ", "empty.txt"], 0, "", ""),
    (
        &["agree", "same.txt", "cr.txt"],
        1,
        "same.txt: lf=2 go=2 rust=2 universal=2 js=2 python=2 unicode=2 agree\ncr.txt: lf=1 go=1 rust=1 universal=2 js=2 python=2 unicode=2 differ\n",
        "",
    ),
    (
        &[
            "agree",
            "--differ",
            "same.txt",
            "cr.txt",
            "empty.txt",
            "ls.txt",
        ],
        1,
        "cr.txt: lf=1 go=1 rust=1 universal=2 js=2 python=2 unicode=2 differ\nls.txt: lf=1 go=1 rust=1 universal=1 js=2 python=2 unicode=2 differ\n",
        "",
    ),
    (&["agree", "cr.txt", "--differ", "same.txt"], 1, CR, ""),
    (&["agree", "same.txt", "--differ"], 0, "", ""),
    (&["agree", "--differ", "--differ", "cr.txt"], 1, CR, ""),
    (&["agree", "--differ", "-"], 1, STDIN, ""),
    (&["agree", "--differ", "ls.txt"], 1, LS, ""),
    (
        &["agree", "--differ"],
        2,
        "",
        "eolscope: usage: agree [--differ] FILE...\n",
    ),
    (
        &["agree", "--differ=1", "cr.txt"],
        2,
        "",
        "eolscope: unknown option \"--differ=1\"\n",
    ),
    (
        &["agree", "--diff", "cr.txt"],
        2,
        "",
        "eolscope: unknown option \"--diff\"\n",
    ),
    (
        &["agree", "--differ", "missing.txt"],
        2,
        "",
        "eolscope: missing.txt: cannot read the file\n",
    ),
    (
        &["agree", "--differ", "cr.txt", "missing.txt"],
        2,
        "",
        "eolscope: missing.txt: cannot read the file\n",
    ),
    (
        &["agree", "--differ", "-", "-"],
        2,
        "",
        "eolscope: standard input is named twice\n",
    ),
    (
        &["kinds", "--differ", "cr.txt"],
        2,
        "",
        "eolscope: unknown option \"--differ\"\n",
    ),
    (&["agree", "same.txt"], 0, SAME, ""),
];

#[test]
fn agree_differ_rows() {
    for (args, status, out, err) in ROWS {
        let io = FakeIo {
            out: RefCell::new(String::new()),
            err: RefCell::new(String::new()),
        };
        let argv: Vec<String> = args.iter().map(|word| word.to_string()).collect();

        assert_eq!(cli::run(&argv, &io), *status, "{args:?}");
        assert_eq!(*io.out.borrow(), *out, "{args:?}");
        assert_eq!(*io.err.borrow(), *err, "{args:?}");
    }
}
