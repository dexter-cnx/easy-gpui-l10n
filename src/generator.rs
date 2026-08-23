use crate::loader::parse_csv;
use anyhow::{anyhow, Context, Result};
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Ftl,
    Both,
}

/// Generate localization artifacts from a CSV source of truth.
///
/// JSON output: `{out_dir}/locales/{lang}/app.json`
/// FTL output:  `{out_dir}/{lang}/main.ftl`
pub fn generate_from_csv(csv_path: &str, out_dir: &str, format: Format) -> Result<()> {
    let csv_content = fs::read_to_string(csv_path)
        .with_context(|| format!("failed to read translations CSV: {csv_path}"))?;
    let translations = parse_csv(&csv_content)?;
    let out_dir = Path::new(out_dir);

    for (locale, messages) in translations {
        if matches!(format, Format::Json | Format::Both) {
            let mut root = Value::Object(Map::new());
            let mut keys: Vec<_> = messages.keys().collect();
            keys.sort();
            for key in keys {
                set_nested(&mut root, key, Value::String(messages[key].clone()))?;
            }

            let dir = out_dir.join("locales").join(&locale);
            fs::create_dir_all(&dir)
                .with_context(|| format!("failed to create {}", dir.display()))?;
            let path = dir.join("app.json");
            let json = serde_json::to_string_pretty(&root)?;
            fs::write(&path, format!("{json}\n"))
                .with_context(|| format!("failed to write {}", path.display()))?;
        }

        if matches!(format, Format::Ftl | Format::Both) {
            let dir = out_dir.join(&locale);
            fs::create_dir_all(&dir)
                .with_context(|| format!("failed to create {}", dir.display()))?;
            let path = dir.join("main.ftl");

            let mut keys: Vec<_> = messages.keys().collect();
            keys.sort();
            let mut ftl = String::new();
            for key in keys {
                let fluent_key = to_kebab_case(key);
                let fluent_value = to_fluent_placeholders(&messages[key]);
                ftl.push_str(&fluent_key);
                ftl.push_str(" = ");
                ftl.push_str(&fluent_value);
                ftl.push('\n');
            }
            fs::write(&path, ftl)
                .with_context(|| format!("failed to write {}", path.display()))?;
        }
    }

    Ok(())
}

pub(crate) fn set_nested(root: &mut Value, dotted_key: &str, value: Value) -> Result<()> {
    let parts: Vec<&str> = dotted_key.split('.').filter(|part| !part.is_empty()).collect();
    if parts.is_empty() {
        return Err(anyhow!("translation key cannot be empty"));
    }

    let mut current = root;
    for (index, part) in parts.iter().enumerate() {
        let is_last = index == parts.len() - 1;
        let object = current
            .as_object_mut()
            .ok_or_else(|| anyhow!("key collision while generating `{dotted_key}`"))?;

        if is_last {
            if object.contains_key(*part) {
                return Err(anyhow!("duplicate/colliding key `{dotted_key}`"));
            }
            object.insert((*part).to_owned(), value);
            return Ok(());
        }

        current = object
            .entry((*part).to_owned())
            .or_insert_with(|| Value::Object(Map::new()));

        if !current.is_object() {
            return Err(anyhow!("key collision while generating `{dotted_key}`"));
        }
    }

    Ok(())
}

fn to_kebab_case(key: &str) -> String {
    let mut out = String::with_capacity(key.len());
    let mut previous_dash = false;
    for ch in key.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            previous_dash = false;
        } else if !previous_dash && !out.is_empty() {
            out.push('-');
            previous_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

fn to_fluent_placeholders(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    let mut out = String::with_capacity(value.len() + 8);
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '{' {
            if let Some(relative_end) = chars[i + 1..].iter().position(|ch| *ch == '}') {
                let end = i + 1 + relative_end;
                let inner: String = chars[i + 1..end].iter().collect();
                let trimmed = inner.trim();
                if is_placeholder_name(trimmed) {
                    out.push_str("{ $");
                    out.push_str(trimmed);
                    out.push_str(" }");
                    i = end + 1;
                    continue;
                }
                if let Some(name) = trimmed.strip_prefix('$').map(str::trim) {
                    if is_placeholder_name(name) {
                        out.push_str("{ $");
                        out.push_str(name);
                        out.push_str(" }");
                        i = end + 1;
                        continue;
                    }
                }
            }
        }

        out.push(chars[i]);
        i += 1;
    }

    out
}

fn is_placeholder_name(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first == '_' || first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|ch| ch == '_' || ch == '-' || ch.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kebab_conversion() {
        assert_eq!(to_kebab_case("settings.title"), "settings-title");
        assert_eq!(to_kebab_case("dialog.delete_title"), "dialog-delete-title");
    }

    #[test]
    fn fluent_placeholder_conversion() {
        assert_eq!(
            to_fluent_placeholders("Delete {name}?"),
            "Delete { $name }?"
        );
        assert_eq!(to_fluent_placeholders("Hi { $name }"), "Hi { $name }");
    }

    #[test]
    fn generates_both_formats() {
        let temp = tempfile::tempdir().unwrap();
        let csv_path = temp.path().join("translations.csv");
        fs::write(
            &csv_path,
            "key,en,th\nsettings.title,Settings,ตั้งค่า\ndialog.delete.title,Delete {name}?,ลบ {name}?\n",
        )
        .unwrap();

        generate_from_csv(
            csv_path.to_str().unwrap(),
            temp.path().to_str().unwrap(),
            Format::Both,
        )
        .unwrap();

        let json = fs::read_to_string(temp.path().join("locales/en/app.json")).unwrap();
        assert!(json.contains("\"settings\""));
        assert!(json.contains("\"title\": \"Settings\""));

        let ftl = fs::read_to_string(temp.path().join("en/main.ftl")).unwrap();
        assert!(ftl.contains("settings-title = Settings"));
        assert!(ftl.contains("dialog-delete-title = Delete { $name }?"));
    }
}
