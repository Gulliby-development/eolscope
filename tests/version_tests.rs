use eolscope::cli::{self, USAGE, VERSION};
use std::process::Command;

struct FakeIo {
    out: std::cell::RefCell<String>,
    err: std::cell::RefCell<String>,
}

impl eolscope::io::Io for FakeIo {
    fn read_stdin(&self) -> Result<Vec<u8>, String> {
        Err("standard input is not used".to_string())
    }

    fn read_file(&self, _path: &str) -> Result<Vec<u8>, String> {
        Err("no such file".to_string())
    }

    fn out(&self, text: &str) {
        self.out.borrow_mut().push_str(text);
    }

    fn err(&self, text: &str) {
        self.err.borrow_mut().push_str(text);
    }
}

fn fake_io() -> FakeIo {
    FakeIo {
        out: Default::default(),
        err: Default::default(),
    }
}

#[test]
fn version_constant_is_the_package_version() {
    assert_eq!(VERSION, "0.1.0");
}

#[test]
fn version_is_printed_with_the_program_name() {
    let io = fake_io();

    assert_eq!(cli::run(&["--version".to_string()], &io), 0);
    assert_eq!(io.out.borrow().as_str(), "eolscope 0.1.0\n");
    assert_eq!(io.err.borrow().as_str(), "");
}

#[test]
fn version_with_another_argument_is_an_unknown_command() {
    let io = fake_io();

    assert_eq!(
        cli::run(&["--version".to_string(), "lines".to_string()], &io),
        2
    );
    assert_eq!(
        cli::run(&["--help".to_string(), "--version".to_string()], &io),
        2
    );
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(
        io.err.borrow().as_str(),
        "eolscope: unknown command \"--version\"\neolscope: unknown command \"--help\"\n"
    );
}

#[test]
fn other_spellings_of_the_version_option_are_unknown_commands() {
    let io = fake_io();

    assert_eq!(cli::run(&["-V".to_string()], &io), 2);
    assert_eq!(cli::run(&["--Version".to_string()], &io), 2);
    assert_eq!(cli::run(&["--version=1".to_string()], &io), 2);
    assert_eq!(io.out.borrow().as_str(), "");
    assert_eq!(
        io.err.borrow().as_str(),
        "eolscope: unknown command \"-V\"\neolscope: unknown command \"--Version\"\neolscope: unknown command \"--version=1\"\n"
    );
}

#[test]
fn help_does_not_list_the_version_option() {
    let io = fake_io();

    assert_eq!(cli::run(&["--help".to_string()], &io), 0);
    assert_eq!(io.out.borrow().as_str(), USAGE);
    assert!(!USAGE.contains("--version"));
}

#[test]
fn version_is_printed_by_the_installed_program() {
    let output = Command::new(env!("CARGO_BIN_EXE_eolscope"))
        .arg("--version")
        .output()
        .expect("the program runs");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"eolscope 0.1.0\n");
    assert_eq!(output.stderr, b"");
}
