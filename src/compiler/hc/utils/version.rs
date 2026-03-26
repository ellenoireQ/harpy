use std::{env, process::Command};

pub fn get_local_git_ver() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .unwrap();

    let git_hash = String::from_utf8(output.stdout).unwrap();
    git_hash
}

pub fn get_version() {
    let local = get_local_git_ver();
    let proj_ver = env!("CARGO_PKG_VERSION");
    println!("hc (harpy-compiler) {} {}", proj_ver, local)
}
