use std::io::{Read, Write};

struct RealIo;

impl eolscope::io::Io for RealIo {
    fn read_stdin(&self) -> Result<Vec<u8>, String> {
        let mut input = Vec::new();
        std::io::stdin()
            .read_to_end(&mut input)
            .map(|_| input)
            .map_err(|error| error.to_string())
    }

    fn read_file(&self, path: &str) -> Result<Vec<u8>, String> {
        std::fs::read(path).map_err(|error| error.to_string())
    }

    fn out(&self, text: &str) {
        let mut stdout = std::io::stdout().lock();
        let _ = stdout.write_all(text.as_bytes());
        let _ = stdout.flush();
    }

    fn err(&self, text: &str) {
        let mut stderr = std::io::stderr().lock();
        let _ = stderr.write_all(text.as_bytes());
        let _ = stderr.flush();
    }
}

fn main() {
    let args = std::env::args_os()
        .skip(1)
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let io = RealIo;
    std::process::exit(eolscope::cli::run(&args, &io));
}
