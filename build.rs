
use std::process::Command;

#[cfg(windows)]
fn main() {
    let version = git_semver();
    let hash = git_hash();
    println!("cargo:rustc-env=GIT_PKG_VERSION_SEMVER={}", version);

    let mut res = winres::WindowsResource::new();
    res.set_icon("app.ico")
        .set_language(0x0409)
        .set("FileDescription", &format!("Lyche {}", hash));
    res.compile().unwrap();
}

fn git_semver() -> String {
    let output = Command::new("git")
        .args(&["describe", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn git_hash() -> String {
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

#[cfg(unix)]
fn main() {}
