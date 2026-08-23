//! Optional GPUI integration.

use crate::{set_locale, tr, tr_with_args};
use ::gpui::{App, Global};
use std::collections::HashMap;

/// GPUI global carrying the locale used by the application UI.
///
/// The actual translation table remains in the crate's thread-safe global core,
/// so this type stays intentionally small and cheap to update.
#[derive(Debug, Clone)]
pub struct I18nGlobal {
    locale: String,
}

impl I18nGlobal {
    pub fn new(locale: impl Into<String>) -> Self {
        let locale = locale.into();
        set_locale(&locale);
        Self { locale }
    }

    pub fn locale(&self) -> &str {
        &self.locale
    }

    pub fn set_locale(&mut self, locale: impl Into<String>) {
        self.locale = locale.into();
        set_locale(&self.locale);
    }

    pub fn tr(&self, key: &str) -> String {
        set_locale(&self.locale);
        tr(key)
    }

    pub fn tr_with_args(&self, key: &str, args: HashMap<String, String>) -> String {
        set_locale(&self.locale);
        tr_with_args(key, args)
    }

    /// Install or replace the GPUI global and refresh all windows.
    pub fn install(cx: &mut App, locale: impl Into<String>) {
        cx.set_global(Self::new(locale));
        cx.refresh_windows();
    }

    /// Update the active locale stored in GPUI and refresh all windows.
    pub fn set_app_locale(cx: &mut App, locale: impl Into<String>) {
        let locale = locale.into();
        if let Some(global) = cx.try_global::<Self>() {
            if global.locale == locale {
                return;
            }
        }

        cx.set_global(Self::new(locale));
        cx.refresh_windows();
    }

    /// Explicitly request a redraw of every GPUI window.
    pub fn refresh(cx: &mut App) {
        cx.refresh_windows();
    }
}

impl Global for I18nGlobal {}
