const README: &str = include_str!("../README.md");

#[test]
fn install_heading_is_present() {
    assert!(README.lines().any(|line| line == "## Install"));
}

#[test]
fn usage_heading_is_present() {
    assert!(README.lines().any(|line| line == "## Usage"));
}

#[test]
fn commands_heading_is_present() {
    assert!(README.lines().any(|line| line == "## Commands"));
}

#[test]
fn development_heading_is_present() {
    assert!(README.lines().any(|line| line == "## Development"));
}

#[test]
fn commands_section_lists_every_exact_usage_line() {
    let section = README
        .split_once("## Commands\n")
        .map(|(_, rest)| rest.split("\n## ").next().unwrap_or(""));
    assert!(section.is_some());
    let section = section.unwrap_or("");

    for usage in [
        "usage: eolscope lines PROFILE FILE",
        "usage: eolscope agree FILE...",
        "usage: eolscope kinds FILE...",
        "usage: eolscope where OFFSET FILE",
        "usage: eolscope trim [--all] PROFILE FILE...",
        "usage: eolscope chars [CODE...]",
        "usage: eolscope convert [--final-newline] [--strip-bom] FROM TO FILE",
        "usage: eolscope check POLICY FILE...",
    ] {
        assert!(section.lines().any(|line| line == usage));
    }
}
