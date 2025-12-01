use std::{fs, path::PathBuf, process::Command};

use embed_manifest::{
    embed_manifest, empty_manifest,
    manifest::{DpiAwareness::*, MaxVersionTested::*, SupportedOS::*},
};

fn main() {
    let manifest = empty_manifest()
        .name(env!("CARGO_PKG_NAME"))
        .dpi_awareness(PerMonitorV2Only)
        .supported_os(Windows10..)
        .max_version_tested(Windows10Version1903);

    embed_manifest(manifest).expect("Fail to embed manifest");

    println!("cargo:rerun-if-changed=build.rs");

    check_i18n();
}

fn check_i18n() {
    let todo_file = "i18n/TODO.yml";

    fs::remove_file(todo_file).ok();
    Command::new("cargo").args(&["i18n"]).status().ok();
    PathBuf::from(todo_file).exists().then(|| {
        panic!("Please resolve all TODOs in {todo_file} before building.");
    });

    println!("cargo:rerun-if-changed=config.toml");
    println!("cargo:rerun-if-changed={todo_file}");
}
