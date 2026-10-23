<p align="center"><img src="docs/logo.svg" width="96" alt="" /></p>

# eolscope

An offline command-line tool that shows where editors, `git`, `wc` and the standard libraries of Python, JavaScript, Go, Rust and Java disagree about where a line ends and what counts as trailing whitespace.

[![CI](https://github.com/Gulliby-development/eolscope/actions/workflows/ci.yml/badge.svg)](https://github.com/Gulliby-development/eolscope/actions/workflows/ci.yml) [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Why

A U+2028 inside a JSON string is one line for `wc -l` and two for Python's `str.splitlines()`; a trailing U+00A0 survives `trim()` in Java and not in Python. eolscope cuts a file into lines under seven named profiles, locates a character under each of them, compares five trimming rules on every line, converts line endings and checks a file against a small policy. It is written in Rust with the standard library only, reads only the files you name (or standard input for `-`) and makes no network calls.

The examples below describe short UTF-8 fixtures rather than relying on shell-specific file-creation syntax. In fixture descriptions, `<LF>`, `<CR>` and `<TAB>` denote the corresponding single characters, while tokens such as `<U+0085>` and `<U+2028>` denote one character with that code point; NBSP, BOM and other visible Unicode characters are written directly where shown. Output examples are the program's real output. The `convert` output is written in the same notation.

## Install

Install Rust and Cargo with a toolchain that supports the package's Rust 2024 edition. The package has no third-party dependencies and builds offline once the Rust toolchain is available.

From a checkout of this repository, install the executable locally with:

```sh
cargo install --path .
```

Cargo places installed binaries in its user-level binary directory; ensure that directory is on `PATH`. The tool supports Windows, macOS and Linux. To see the command list and syntax, run:

```sh
eolscope --help
```

When working directly from the checkout, `cargo run -- --help` displays the same help text.

## Usage

Each example assumes a UTF-8 file named `t.txt` with the explicitly described contents. The examples use the installed `eolscope` command.

### List lines

Fixture: `t.txt` contains `a<CR><LF>b<CR><LF><CR>c<CR>`.

```sh
eolscope lines go t.txt
```

Output:

```text
LINE  END   CHARS  TEXT
1     CRLF  1      "a"
2     CRLF  1      "b"
3     none  2      "?c"
```

### Compare line counts

Fixture: `t.txt` contains `a<CR>b`.

```sh
eolscope agree t.txt
```

Output:

```text
t.txt: lf=1 go=1 rust=1 universal=2 js=2 python=2 unicode=2 differ
```

### Inventory terminators

Fixture: `t.txt` contains `a<LF>b<CR><LF>c<U+0085>d<U+2028>e<U+2029>`.

```sh
eolscope kinds t.txt
```

Output:

```text
t.txt:
  lines  5
  LF     1
  CRLF   1
  NEL    1
  LS     1
  PS     1
  style  mixed
  final  yes
  bom    no
```

### Locate a character

Fixture: `t.txt` contains `ab<U+2028>c<CR><LF>d<U+0085>e`. Offset `5` is a Unicode-scalar offset, not a byte or UTF-16 offset.

```sh
eolscope where 5 t.txt
```

Output:

```text
PROFILE    LINE  COL  AT
lf         1     6    LF
go         1     5    CRLF
rust       1     5    CRLF
universal  1     5    CRLF
js         2     2    CRLF
python     2     2    CRLF
unicode    2     2    CRLF
```

### Compare trailing-whitespace rules

Fixture: `t.txt` contains `a <LF>b <LF>c﻿<LF>d　<LF>e<TAB> <LF>f<U+0085><LF>g<LF>` (the NBSP, BOM, IDEOGRAPHIC SPACE and NEL are intentional).

```sh
eolscope trim lf t.txt
```

Output:

```text
FILE   LINE  js-trim  py-strip  rs-trim  java-trim  java-strip  VERDICT
t.txt  2     1        1         1        0          0           differ
t.txt  3     1        0         0        0          0           differ
t.txt  4     1        1         1        0          1           differ
t.txt  5     2        2         2        0          0           differ
t.txt  6     0        1         1        0          0           differ
```

### Inspect character rules

No file is read for this command. This invocation asks about NEL, paragraph separator, an ordinary letter and an astral character:

```sh
eolscope chars U+0085 2029 0041 u+1F600
```

Output:

```text
CODE     NAME  lf  go  rust  universal  js  python  unicode  js-trim  py-strip  rs-trim  java-trim  java-strip
U+0085   NEL   .   .   .     .          .   T       T        .        x         x        .          .
U+2029   PS    .   .   .     .          T   T       T        x        x         x        .          x
U+0041   -     .   .   .     .          .   .       .        .        .         .        .          .
U+1F600  -     .   .   .     .          .   .       .        .        .         .        .          .
```

### Convert line endings

Fixture: `t.txt` contains `a<LF>b<LF>`. The converter writes only the converted text; the output below uses the specification's notation.

```sh
eolscope convert lf crlf t.txt
```

Output: `a<CR><LF>b<CR><LF>`

### Check a policy

Fixture: `t.txt` contains `a<LF>b<CR><LF>c<U+2028>d<LF>`.

```sh
eolscope check lf t.txt
```

Output:

```text
t.txt:2:2: EV001 CRLF terminator, expected LF
t.txt:3:2: EV001 LS terminator, expected LF
2 problems
```

## Commands

The help output presents these exact usage lines in this order:

```text
usage: eolscope lines PROFILE FILE
usage: eolscope agree FILE...
usage: eolscope kinds FILE...
usage: eolscope where OFFSET FILE
usage: eolscope trim [--all] PROFILE FILE...
usage: eolscope chars [CODE...]
usage: eolscope convert [--final-newline] [--strip-bom] FROM TO FILE
usage: eolscope check POLICY FILE...
```

The seven case-sensitive line profiles are `lf`, `go`, `rust`, `universal`, `js`, `python` and `unicode`. They model different line-boundary conventions; for example, `lf` treats only LF as a terminator, while `python` also recognizes VT, FF, FS, GS, RS, NEL, LS and PS. CRLF is treated as one terminator where both CR and LF end lines. The `go` and `rust` profiles also model their special handling of CR before LF, and `go` drops a final CR from an unterminated line.

The six case-sensitive conversion styles are `lf`, `crlf`, `cr`, `nel`, `ls` and `ps`. `convert` changes recognized line terminators only; other characters remain unchanged. By default it preserves whether the final line was terminated. `--final-newline` adds a terminator to a nonempty unterminated final line, and `--strip-bom` removes one leading byte-order mark before conversion. BOMs are otherwise preserved as ordinary text.

The five whitespace rules are `js-trim`, `py-strip`, `rs-trim`, `java-trim` and `java-strip`. `trim` compares the number of trailing characters each rule would strip from every selected line; it lists disagreements by default and includes every line with `--all`. `chars` shows which line profiles and whitespace rules recognize selected Unicode code points. Code values use four to six hexadecimal digits, optionally prefixed by `U+` or `u+`.

Command reference:

- `lines PROFILE FILE` lists line number, terminator, content length and displayed content for one file.
- `agree FILE...` compares line counts across all seven profiles for each file.
- `kinds FILE...` inventories terminator kinds, style, final-terminator state and leading BOM state.
- `where OFFSET FILE` locates a Unicode scalar offset under all profiles. Offsets are canonical decimal, count Unicode scalar values, and may be at most nine digits.
- `trim [--all] PROFILE FILE...` compares the five trailing-whitespace rules for each line.
- `chars [CODE...]` reports line-ending and whitespace-rule membership for the supplied code points, or the built-in special-character list when no codes are supplied.
- `convert [--final-newline] [--strip-bom] FROM TO FILE` converts recognized terminators from profile `FROM` to style `TO`; the converted text is written raw to standard output.
- `check POLICY FILE...` checks files against comma-separated policy items. A policy may name one line style (`lf`, `crlf`, `cr`, `nel`, `ls` or `ps`), `final` or `nofinal`, `bom` or `nobom`, and/or `notrail`. Repeated compatible items are allowed; opposing settings or different styles conflict. `notrail` uses the `rs-trim` whitespace rule.

Files must be valid UTF-8: malformed byte sequences are rejected rather than replaced. Use `-` as a file argument to read standard input; naming `-` twice is an error. A leading BOM is kept by inspection commands and by conversion unless `--strip-bom` is supplied.

Exit status is `0` for success, `1` for a negative result, and `2` for an error. The negative-result commands are `agree` (profiles disagree), `kinds` (a file has mixed terminators), `trim` (rules disagree on a line) and `check` (a policy problem is found). On an error, standard output is empty and standard error contains exactly one diagnostic line; commands that read multiple files do not emit partial report output if a later input fails. Conversion is raw text output and does not add a report or diagnostic wrapper to the converted data. Do not redirect conversion output onto its input file, since the output stream could overwrite the source; choose a separate destination file.

## Development

Run the project checks from the repository root:

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

These checks use the Rust toolchain and the project's standard-library-only dependencies.

## License

MIT, see [LICENSE](LICENSE).

---

Built with BAKED
