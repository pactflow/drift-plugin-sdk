//! Reads the version of the WIT package from the vendored `wit/plugin.wit`, so that what the SDK
//! reports as the WIT it targets, and as a plugin's `interface-version`, cannot disagree with it.

use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wit/plugin.wit");
    println!("cargo:rerun-if-changed=build.rs");

    let wit = std::fs::read_to_string("wit/plugin.wit").expect("cannot read wit/plugin.wit");
    let version = wit
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("package drift:plugins@")?
                .strip_suffix(';')
        })
        .expect("wit/plugin.wit has no `package drift:plugins@<version>;` line");
    let major: u16 = version
        .split('.')
        .next()
        .and_then(|major| major.parse().ok())
        .unwrap_or_else(|| panic!("`{version}` does not start with a major version number"));

    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is set by cargo"));
    std::fs::write(
        out.join("wit_version.rs"),
        format!(
            "/// The version of the `drift:plugins` WIT package this SDK targets.\n\
             pub const WIT_VERSION: &str = {version:?};\n\n\
             /// The major version of [`WIT_VERSION`]. It is what a plugin's manifest reports as its\n\
             /// `interface-version`, and what Drift checks.\n\
             pub const INTERFACE_VERSION: u16 = {major};\n"
        ),
    )
    .expect("cannot write wit_version.rs");
}
