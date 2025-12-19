use resvg::{tiny_skia, usvg};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use embed_manifest::{
    embed_manifest, empty_manifest,
    manifest::{DpiAwareness::*, MaxVersionTested::*, SupportedOS::*},
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    gen_manifest();
    check_i18n();
    let ico = generate_ico(&out_dir);
    embed_icon(ico.to_str().unwrap());
    strip_commandline();

    println!("cargo:rerun-if-changed={}", ico.to_string_lossy());
}

fn gen_manifest() {
    let manifest = empty_manifest()
        .name(env!("CARGO_PKG_NAME"))
        .dpi_awareness(PerMonitorV2Only)
        .supported_os(Windows10..)
        .max_version_tested(Windows10Version1903);
    embed_manifest(manifest).expect("Fail to embed manifest");
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

fn svg2bitmap<S: AsRef<str>>(svg: S) -> (Vec<u8>, u32, u32) {
    let tree = usvg::Tree::from_str(svg.as_ref(), &Default::default()).unwrap();

    let (w, h) = {
        let size = tree.size();
        (size.width() as u32, size.height() as u32)
    };

    let mut pixmap = tiny_skia::Pixmap::new(w, h).unwrap();

    resvg::render(&tree, Default::default(), &mut pixmap.as_mut());

    (pixmap.data().to_vec(), w, h)
}

fn generate_ico<P: AsRef<Path>>(dir: P) -> PathBuf {
    let svg_path = PathBuf::from("assets/nexus.logo.svg");
    let svg = fs::read_to_string(&svg_path).unwrap();
    let (data, w, h) = svg2bitmap(svg);
    let image_buf = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(w, h, data).unwrap();

    let icon_path = dir
        .as_ref()
        .join(svg_path.file_name().unwrap())
        .with_extension("ico");

    image_buf.save(&icon_path).unwrap();

    icon_path
}

include!("src/resource.rs");
fn embed_icon(icon_path: &str) {
    winres::WindowsResource::new()
        .set_icon(icon_path)
        .set_icon_with_id(icon_path, &APP_ICON.to_string())
        .compile()
        .expect("Failed to embed icon");
    println!("cargo:rerun-if-changed=logo.ico");
}


fn strip_commandline() {
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() && std::env::var("PROFILE").unwrap() == "release" {
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
        println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
    }
}