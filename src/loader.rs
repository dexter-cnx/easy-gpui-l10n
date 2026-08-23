use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;

pub(crate) fn parse_csv(csv_content: &str) -> Result<HashMap<String, HashMap<String, String>>> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(csv_content.as_bytes());

    let headers = reader
        .headers()
        .context("failed to read CSV headers")?
        .clone();
    if headers.len() < 2 {
        return Err(anyhow!(
            "translations CSV must contain `key` plus at least one locale column"
        ));
    }
    if headers.get(0) != Some("key") {
        return Err(anyhow!("first CSV column must be named `key`"));
    }

    let locales: Vec<String> = headers.iter().skip(1).map(str::to_owned).collect();
    let mut translations: HashMap<String, HashMap<String, String>> = locales
        .iter()
        .cloned()
        .map(|locale| (locale, HashMap::new()))
        .collect();

    for (row_index, row) in reader.records().enumerate() {
        let row = row.with_context(|| format!("invalid CSV record at row {}", row_index + 2))?;
        let key = row.get(0).unwrap_or_default().trim();
        if key.is_empty() {
            continue;
        }

        for (column_index, locale) in locales.iter().enumerate() {
            let value = row.get(column_index + 1).unwrap_or_default().trim();
            if value.is_empty() {
                continue;
            }
            translations
                .get_mut(locale)
                .expect("locale map initialized from headers")
                .insert(key.to_owned(), value.to_owned());
        }
    }

    Ok(translations)
}
