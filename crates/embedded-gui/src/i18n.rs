//! Zero-allocation internationalization (i18n) and localization for `#![no_std]` embedded targets.
//! Allows microcontrollers to store compact string translation tables in Flash and switch languages
//! dynamically at runtime without heap allocation.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LanguageId(pub u8);

impl LanguageId {
    pub const EN: Self = Self(0);
    pub const ES: Self = Self(1);
    pub const DE: Self = Self(2);
    pub const FR: Self = Self(3);
    pub const IT: Self = Self(4);
    pub const JA: Self = Self(5);
    pub const ZH: Self = Self(6);

    pub const fn new(id: u8) -> Self {
        Self(id)
    }

    pub const fn raw(self) -> u8 {
        self.0
    }
}

/// A single translation entry mapping a string key to localized strings across language indices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TranslationEntry<'a> {
    pub key: &'a str,
    pub translations: &'a [&'a str],
}

impl<'a> TranslationEntry<'a> {
    pub const fn new(key: &'a str, translations: &'a [&'a str]) -> Self {
        Self { key, translations }
    }

    #[inline]
    pub fn get(&self, lang: LanguageId) -> Option<&'a str> {
        let idx = lang.0 as usize;
        if idx < self.translations.len() {
            Some(self.translations[idx])
        } else {
            None
        }
    }
}

/// A zero-allocation static translation table stored in ROM/Flash.
#[derive(Clone, Copy, Debug)]
pub struct TranslationTable<'a> {
    pub entries: &'a [TranslationEntry<'a>],
    pub active_lang: LanguageId,
    pub fallback_lang: LanguageId,
}

impl<'a> TranslationTable<'a> {
    pub const fn new(entries: &'a [TranslationEntry<'a>]) -> Self {
        Self {
            entries,
            active_lang: LanguageId::EN,
            fallback_lang: LanguageId::EN,
        }
    }

    pub const fn with_active_lang(mut self, lang: LanguageId) -> Self {
        self.active_lang = lang;
        self
    }

    pub fn set_language(&mut self, lang: LanguageId) {
        self.active_lang = lang;
    }

    /// Looks up a localized string by key using the active language, falling back to fallback_lang or the raw key.
    pub fn translate(&self, key: &'a str) -> &'a str {
        if let Some(entry) = self.entries.iter().find(|e| e.key == key) {
            if let Some(s) = entry.get(self.active_lang) {
                return s;
            }
            if let Some(s) = entry.get(self.fallback_lang) {
                return s;
            }
        }
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static ENTRIES: [TranslationEntry<'static>; 3] = [
        TranslationEntry::new("btn_start", &["Start", "Iniciar", "Starten"]),
        TranslationEntry::new("btn_stop", &["Stop", "Detener", "Stoppen"]),
        TranslationEntry::new("status_ready", &["Ready", "Listo", "Bereit"]),
    ];

    #[test]
    fn test_translation_table_lookup() {
        let mut tr = TranslationTable::new(&ENTRIES);

        assert_eq!(tr.translate("btn_start"), "Start");
        assert_eq!(tr.translate("btn_stop"), "Stop");

        // Switch to Spanish
        tr.set_language(LanguageId::ES);
        assert_eq!(tr.translate("btn_start"), "Iniciar");
        assert_eq!(tr.translate("btn_stop"), "Detener");
        assert_eq!(tr.translate("status_ready"), "Listo");

        // Switch to German
        tr.set_language(LanguageId::DE);
        assert_eq!(tr.translate("btn_start"), "Starten");
        assert_eq!(tr.translate("status_ready"), "Bereit");

        // Fallback for missing key
        assert_eq!(tr.translate("unknown_key"), "unknown_key");
    }
}
