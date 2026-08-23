mod generator;
mod loader;

#[cfg(feature = "gpui")]
pub mod gpui;

pub use generator::{generate_from_csv, Format};

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Default)]
struct I18nState {
    translations: HashMap<String, HashMap<String, String>>,
    locale: String,
    fallback: String,
}

static STATE: Lazy<RwLock<I18nState>> = Lazy::new(|| RwLock::new(I18nState::default()));

/// Initialize the global localization state from CSV content.
///
/// The first CSV column must be `key`; all following headers are locale names.
/// Empty cells are ignored and therefore fall back to the configured fallback locale.
pub fn init_from_csv(csv_content: &str, fallback: &str) -> Result<()> {
    let translations = loader::parse_csv(csv_content)?;

    if !translations.contains_key(fallback) {
        return Err(anyhow!(
            "fallback locale `{fallback}` does not exist in CSV headers"
        ));
    }

    let mut state = STATE
        .write()
        .map_err(|_| anyhow!("localization state lock poisoned"))?;
    state.translations = translations;
    state.fallback = fallback.to_owned();
    state.locale = fallback.to_owned();
    Ok(())
}

/// Change the active locale.
///
/// This intentionally never panics. Unknown locales are allowed and will resolve
/// through the configured fallback locale until translations for them are loaded.
pub fn set_locale(lang: &str) {
    if let Ok(mut state) = STATE.write() {
        state.locale = lang.to_owned();
    }
}

/// Return the currently selected locale.
pub fn locale() -> String {
    STATE
        .read()
        .map(|state| state.locale.clone())
        .unwrap_or_default()
}

/// Translate `key`. Missing translations fall back to the fallback locale;
/// if no translation exists there either, the key itself is returned.
pub fn tr(key: &str) -> String {
    tr_with_args(key, HashMap::new())
}

/// Translate `key` and replace placeholders.
///
/// Both easy-localize-style `{name}` and Fluent-style `{ $name }` placeholders
/// are supported at runtime.
pub fn tr_with_args(key: &str, args: HashMap<String, String>) -> String {
    let raw = STATE
        .read()
        .ok()
        .and_then(|state| {
            lookup(&state.translations, &state.locale, key)
                .or_else(|| lookup(&state.translations, &state.fallback, key))
                .map(str::to_owned)
        })
        .unwrap_or_else(|| key.to_owned());

    replace_args(raw, &args)
}

fn lookup<'a>(
    translations: &'a HashMap<String, HashMap<String, String>>,
    locale: &str,
    key: &str,
) -> Option<&'a str> {
    translations
        .get(locale)
        .and_then(|messages| messages.get(key))
        .map(String::as_str)
}

fn replace_args(mut value: String, args: &HashMap<String, String>) -> String {
    for (name, replacement) in args {
        for pattern in [
            format!("{{{name}}}"),
            format!("{{${name}}}"),
            format!("{{ ${name}}}"),
            format!("{{${name} }}"),
            format!("{{ ${name} }}"),
        ] {
            value = value.replace(&pattern, replacement);
        }
    }
    value
}

/// Translate using easy_localize-like syntax.
///
/// ```
/// # easy_gpui_l10n::init_from_csv("key,en\nhello,Hello\nwelcome,Hello {name}\n", "en")?;
/// assert_eq!(easy_gpui_l10n::tr!("hello"), "Hello");
/// assert_eq!(easy_gpui_l10n::tr!("welcome", name = "Dexter"), "Hello Dexter");
/// # Ok::<(), anyhow::Error>(())
/// ```
#[macro_export]
macro_rules! tr {
    ($key:expr $(,)?) => {
        $crate::tr($key)
    };
    ($key:expr, $($name:ident = $value:expr),+ $(,)?) => {{
        let mut args = ::std::collections::HashMap::<String, String>::new();
        $(
            args.insert(stringify!($name).to_owned(), ($value).to_string());
        )+
        $crate::tr_with_args($key, args)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    const CSV: &str = "key,en,th,zh-CN\nhello,Hello,สวัสดี,你好\nsettings.title,Settings,ตั้งค่า,设置\ndialog.delete.title,Delete {name}?,ลบ {name}?,删除 {name}？\nfallback.only,Fallback only,,\n";

    #[test]
    fn translates_and_falls_back() {
        init_from_csv(CSV, "en").unwrap();
        set_locale("th");
        assert_eq!(tr("hello"), "สวัสดี");
        assert_eq!(tr("fallback.only"), "Fallback only");
        assert_eq!(tr("missing.key"), "missing.key");
    }

    #[test]
    fn replaces_both_placeholder_styles() {
        init_from_csv(CSV, "en").unwrap();
        let mut args = HashMap::new();
        args.insert("name".into(), "Dexter".into());
        assert_eq!(tr_with_args("dialog.delete.title", args), "Delete Dexter?");

        let mut direct = HashMap::new();
        direct.insert("name".into(), "Dexter".into());
        assert_eq!(replace_args("Hi { $name }".into(), &direct), "Hi Dexter");
    }

    #[test]
    fn macro_supports_named_args() {
        init_from_csv(CSV, "en").unwrap();
        assert_eq!(
            crate::tr!("dialog.delete.title", name = "Dexter"),
            "Delete Dexter?"
        );
    }
}
