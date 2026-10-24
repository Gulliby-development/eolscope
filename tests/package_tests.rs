use std::process::Command;

fn package_files() -> Vec<String> {
    let output = Command::new(env!("CARGO"))
        .args(["package", "--list", "--allow-dirty", "--offline"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo runs");

    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .expect("the listing is UTF-8")
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn package_holds_the_manifest_and_the_lock_file() {
    let files = package_files();

    assert!(files.contains(&"Cargo.toml".to_string()));
    assert!(files.contains(&"Cargo.lock".to_string()));
}

#[test]
fn package_holds_the_license_and_the_readme() {
    let files = package_files();

    assert!(files.contains(&"LICENSE".to_string()));
    assert!(files.contains(&"README.md".to_string()));
}

#[test]
fn package_holds_the_logo_the_readme_shows() {
    assert!(package_files().contains(&"docs/logo.svg".to_string()));
}

#[test]
fn package_holds_the_sources_and_the_tests() {
    let files = package_files();

    assert!(files.contains(&"src/main.rs".to_string()));
    assert!(files.contains(&"src/cli.rs".to_string()));
    assert!(files.contains(&"tests/cli_tests.rs".to_string()));
}

#[test]
fn package_leaves_out_the_github_folder() {
    let files = package_files();

    assert!(!files.iter().any(|file| file.starts_with(".github/")));
}

#[test]
fn package_leaves_out_git_attributes() {
    assert!(!package_files().contains(&".gitattributes".to_string()));
}
