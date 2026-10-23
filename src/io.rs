use std::fmt;

use crate::error::EsError;
use crate::text::plain;

pub trait Io {
    fn read_stdin(&self) -> Result<Vec<u8>, String>;
    fn read_file(&self, path: &str) -> Result<Vec<u8>, String>;
    fn out(&self, text: &str);
    fn err(&self, text: &str);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub display: String,
    pub text: String,
}

pub struct Inputs<'a> {
    io: &'a dyn Io,
    stdin_consumed: bool,
}

impl<'a> Inputs<'a> {
    pub fn new(io: &'a dyn Io) -> Inputs<'a> {
        Inputs {
            io,
            stdin_consumed: false,
        }
    }

    pub fn read(&mut self, arg: &str) -> Result<Source, EsError> {
        if arg == "-" {
            if self.stdin_consumed {
                return Err(EsError("standard input is named twice".to_string()));
            }
            self.stdin_consumed = true;

            let bytes = self
                .io
                .read_stdin()
                .map_err(|_| EsError("<stdin>: cannot read the file".to_string()))?;
            decode(bytes, "<stdin>".to_string())
        } else {
            let display = plain(arg);
            let bytes = self
                .io
                .read_file(arg)
                .map_err(|_| EsError(format!("{display}: cannot read the file")))?;
            decode(bytes, display)
        }
    }
}

impl fmt::Debug for Inputs<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Inputs")
            .field("stdin_consumed", &self.stdin_consumed)
            .finish()
    }
}

impl Clone for Inputs<'_> {
    fn clone(&self) -> Self {
        Self {
            io: self.io,
            stdin_consumed: self.stdin_consumed,
        }
    }
}

impl PartialEq for Inputs<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.io, other.io) && self.stdin_consumed == other.stdin_consumed
    }
}

impl Eq for Inputs<'_> {}

fn decode(bytes: Vec<u8>, display: String) -> Result<Source, EsError> {
    match String::from_utf8(bytes) {
        Ok(text) => Ok(Source { display, text }),
        Err(_) => Err(EsError(format!("{display}: input is not valid UTF-8"))),
    }
}
