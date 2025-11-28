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
}
