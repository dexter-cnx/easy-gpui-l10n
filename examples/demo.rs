use anyhow::Result;
use easy_gpui_l10n::{init_from_csv, set_locale, tr};

const TRANSLATIONS: &str = include_str!("translations.csv");

fn main() -> Result<()> {
    init_from_csv(TRANSLATIONS, "en")?;

    set_locale("en");
    println!("{}", tr!("hello"));
    println!("{}", tr!("dialog.delete.title", name = "Dexter"));

    set_locale("th");
    println!("{}", tr!("hello"));
    println!("{}", tr!("dialog.delete.title", name = "Dexter"));
    println!("{}", tr!("only.english"));

    Ok(())
}
