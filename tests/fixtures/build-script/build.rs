use easy_gpui_l10n::{generate_from_csv, Format};

fn main() {
    println!("cargo:rerun-if-changed=translations.csv");
    generate_from_csv("translations.csv", "generated", Format::Both)
        .expect("build.rs localization generation failed");
}
