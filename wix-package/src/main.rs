use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("Starting Wix installer generation...");
    generate_variable_wxi();
}

fn generate_variable_wxi() {
    let app_name = "LitAudioSinkNexus";

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    println!("Manifest dir: {}", manifest_dir.to_string_lossy());
    let workspace_dir = manifest_dir.parent().unwrap().to_path_buf();
    let wix_dir = manifest_dir.join("wix");
    let wxi_path = wix_dir.join("var.wxi");
    let config_toml = workspace_dir.join("config.toml");

    let exe_file = workspace_dir
        .join("target")
        .join("release")
        .join("nexus.exe");
    let license_file = workspace_dir.join("LICENSE");

    let version = env!("CARGO_PKG_VERSION");
    let upgrade_code =
        uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_X500, version.as_bytes()).to_string();

    let directives = vec![
        ("AppName", app_name),
        ("Manufacturer", env!("CARGO_PKG_AUTHORS")),
        ("Version", env!("CARGO_PKG_VERSION")),
        ("UpgradeCode", &upgrade_code),
        ("AppDes", env!("CARGO_PKG_DESCRIPTION")),
        ("AppExe", exe_file.to_str().unwrap()),
        ("LicenseFile", license_file.to_str().unwrap()),
        ("ConfigFile", config_toml.to_str().unwrap()),
    ];

    let mut wxi_content = String::new();
    for (key, value) in directives {
        wxi_content.push_str(&format!("<?define {}=\"{}\" ?>\n", key, value));
    }

    let wxi_content = format!(
        r#"<Include xmlns="http://wixtoolset.org/schemas/v4/wxs">{}{}</Include>"#,
        "\n", wxi_content
    );

    fs::write(&wxi_path, wxi_content).expect("Unable to write var.wxi file");

    let installer_name = format!("{}-Setup-{}.msi", app_name, version);
    let installer_path = workspace_dir.join("target").join(installer_name);

    Command::new("cargo")
        .args(["build", "--release", "--bin", "nexus"])
        .current_dir(&workspace_dir)
        .status()
        .unwrap();

    let _command = Command::new("wix")
        .args([
            "build",
            "*.wxs",
            "*.wxl",
            "-ext",
            "WixToolset.UI.wixext",
            "-culture",
            "zh-CN",
            "-o",
            installer_path.to_str().unwrap(),
        ])
        .current_dir(&wix_dir)
        .status()
        .unwrap();

    let dummy_config = wix_dir.join("config.toml");
    fs::write(&dummy_config, "# Dummy config file").expect("Unable to write dummy config.toml");
}
