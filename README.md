# easy-gpui-l10n

CSV-first localization for Rust/GPUI, inspired by Flutter `easy_localization`.

- One `translations.csv` source of truth
- Dot-notation keys
- Fallback locale
- `{name}` placeholders at runtime
- JSON generation compatible with `rust-i18n`-style locale trees
- Fluent (`.ftl`) generation with `{ $name }` placeholders
- Optional GPUI integration behind the `gpui` feature
- Thread-safe global runtime state via `once_cell` + `RwLock`

## CSV / ไฟล์ต้นฉบับ

```csv
key,en,th,zh-CN
hello,Hello,สวัสดี,你好
settings.title,Settings,ตั้งค่า,设置
dialog.delete.title,Delete {name}?,ลบ {name}?,删除 {name}？
```

The first column must be `key`. Every other header is treated as a locale. Empty cells are omitted, so lookup falls back to the configured fallback locale.

คอลัมน์แรกต้องชื่อ `key` ส่วนคอลัมน์ถัดไปคือภาษา หาก cell ว่าง crate จะไม่บันทึกค่านั้น และตอน runtime จะ fallback ไปภาษาหลักที่กำหนดไว้

## Runtime usage / การใช้งาน

```rust
use easy_gpui_l10n::{init_from_csv, set_locale, tr};

const TRANSLATIONS: &str = include_str!("translations.csv");

fn main() -> anyhow::Result<()> {
    init_from_csv(TRANSLATIONS, "en")?;

    set_locale("en");
    assert_eq!(tr!("hello"), "Hello");
    assert_eq!(tr!("dialog.delete.title", name = "Dexter"), "Delete Dexter?");

    set_locale("th");
    assert_eq!(tr!("hello"), "สวัสดี");
    assert_eq!(tr!("dialog.delete.title", name = "Dexter"), "ลบ Dexter?");

    Ok(())
}
```

You can also call the functions directly:

```rust
use easy_gpui_l10n::{tr, tr_with_args};
use std::collections::HashMap;

let plain = tr("settings.title");

let mut args = HashMap::new();
args.insert("name".to_string(), "Dexter".to_string());
let text = tr_with_args("dialog.delete.title", args);
```

A missing key never panics; it returns the key itself.

ถ้าหา key ไม่เจอ จะคืนค่า key เดิมกลับมา เช่น `tr!("missing.key") == "missing.key"` และไม่ panic

## Generator

Add the crate as both a normal dependency (if needed at runtime) and a build dependency:

```toml
[dependencies]
easy-gpui-l10n = "0.1"

[build-dependencies]
easy-gpui-l10n = "0.1"
```

### `build.rs`

```rust
use easy_gpui_l10n::{generate_from_csv, Format};

fn main() {
    println!("cargo:rerun-if-changed=translations.csv");

    generate_from_csv(
        "translations.csv",
        "generated",
        Format::Both,
    )
    .expect("failed to generate localization files");
}
```

Generated layout:

```text
generated/
├── locales/
│   ├── en/app.json
│   ├── th/app.json
│   └── zh-CN/app.json
├── en/main.ftl
├── th/main.ftl
└── zh-CN/main.ftl
```

For JSON, `settings.title` becomes:

```json
{
  "settings": {
    "title": "Settings"
  }
}
```

For Fluent, `settings.title` becomes:

```ftl
settings-title = Settings
dialog-delete-title = Delete { $name }?
```

`Format::Json`, `Format::Ftl`, and `Format::Both` are supported. Output directories are created automatically.

## GPUI integration

Enable the optional feature:

```toml
[dependencies]
easy-gpui-l10n = { version = "0.1", features = ["gpui"] }
```

Then install `I18nGlobal` in the GPUI app:

```rust
use easy_gpui_l10n::gpui::I18nGlobal;
use gpui::App;

fn setup(cx: &mut App) {
    I18nGlobal::install(cx, "en");
}
```

Read through the global from your GPUI code:

```rust
let title = cx.global::<I18nGlobal>().tr("settings.title");
```

Change locale and redraw every window:

```rust
I18nGlobal::set_app_locale(cx, "th");
```

`I18nGlobal` only stores the current locale. Translation data stays in the reusable core runtime, keeping the localization layer independent from GPUI and making it suitable for other frontends later.

`I18nGlobal` เก็บเฉพาะ locale ปัจจุบัน ส่วน translation table อยู่ใน core ของ crate ดังนั้น project สามารถใช้ localization เดิมกับ frontend อื่นได้โดยไม่ผูกกับ GPUI

## Demo

```bash
cargo run --example demo
```

Expected output:

```text
Hello
Delete Dexter?
สวัสดี
ลบ Dexter?
English fallback
```

## API

```rust
pub fn init_from_csv(csv_content: &str, fallback: &str) -> anyhow::Result<()>;
pub fn set_locale(lang: &str);
pub fn tr(key: &str) -> String;
pub fn tr_with_args(key: &str, args: HashMap<String, String>) -> String;

pub enum Format { Json, Ftl, Both }
pub fn generate_from_csv(csv_path: &str, out_dir: &str, format: Format) -> anyhow::Result<()>;
```

## License

MIT
