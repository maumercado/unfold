//! Use the app's version (from the root Cargo.toml, which release-please bumps)
//! instead of this crate's own, so About and update checks report the real
//! release number while both apps live side by side.

fn main() {
    let manifest = "../../Cargo.toml";
    println!("cargo:rerun-if-changed={manifest}");
    let text = std::fs::read_to_string(manifest).expect("read root Cargo.toml");
    let version = text
        .lines()
        .skip_while(|l| l.trim() != "[package]")
        .find_map(|l| {
            let rest = l.trim().strip_prefix("version")?.trim().strip_prefix('=')?;
            Some(rest.trim().trim_matches('"').to_string())
        })
        .expect("version in root [package]");
    println!("cargo:rustc-env=UNFOLD_VERSION={version}");
}
