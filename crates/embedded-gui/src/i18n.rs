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

/// A strongly typed numeric identifier for message keys, allowing O(1) indexed lookups.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageId(pub u16);

impl MessageId {
    pub const fn new(id: u16) -> Self {
        Self(id)
    }

    pub const fn raw(self) -> u16 {
        self.0
    }
}

/// A zero-allocation static translation table stored in ROM/Flash.
#[derive(Clone, Copy, Debug)]
pub struct TranslationTable<'a> {
    pub entries: &'a [TranslationEntry<'a>],
    pub active_lang: LanguageId,
    pub fallback_lang: LanguageId,
    pub is_sorted: bool,
}

impl<'a> TranslationTable<'a> {
    pub const fn new(entries: &'a [TranslationEntry<'a>]) -> Self {
        Self {
            entries,
            active_lang: LanguageId::EN,
            fallback_lang: LanguageId::EN,
            is_sorted: false,
        }
    }

    /// Creates a translation table marked as sorted by key, enabling O(log N) binary search lookups.
    pub const fn new_sorted(entries: &'a [TranslationEntry<'a>]) -> Self {
        Self {
            entries,
            active_lang: LanguageId::EN,
            fallback_lang: LanguageId::EN,
            is_sorted: true,
        }
    }

    pub const fn with_active_lang(mut self, lang: LanguageId) -> Self {
        self.active_lang = lang;
        self
    }

    pub const fn with_sorted(mut self, is_sorted: bool) -> Self {
        self.is_sorted = is_sorted;
        self
    }

    pub fn set_language(&mut self, lang: LanguageId) {
        self.active_lang = lang;
    }

    /// Looks up a localized string by key using the active language, falling back to fallback_lang or the raw key.
    pub fn translate(&self, key: &'a str) -> &'a str {
        self.translate_lang(key, self.active_lang)
    }

    /// Looks up a localized string by key for an explicit language, falling back to fallback_lang or the raw key.
    pub fn translate_lang(&self, key: &'a str, lang: LanguageId) -> &'a str {
        let entry = if self.is_sorted {
            self.entries
                .binary_search_by_key(&key, |e| e.key)
                .ok()
                .map(|idx| &self.entries[idx])
        } else {
            self.entries.iter().find(|e| e.key == key)
        };

        if let Some(entry) = entry {
            if let Some(s) = entry.get(lang) {
                return s;
            }
            if let Some(s) = entry.get(self.fallback_lang) {
                return s;
            }
        }
        key
    }

    /// Looks up a localized string by numeric/enum MessageId in O(1) time without string comparisons.
    pub fn translate_id(&self, id: MessageId) -> Option<&'a str> {
        self.translate_id_lang(id, self.active_lang)
    }

    /// Looks up a localized string by numeric/enum MessageId for an explicit language in O(1) time.
    pub fn translate_id_lang(&self, id: MessageId, lang: LanguageId) -> Option<&'a str> {
        let idx = id.0 as usize;
        if let Some(entry) = self.entries.get(idx) {
            if let Some(s) = entry.get(lang) {
                return Some(s);
            }
            if let Some(s) = entry.get(self.fallback_lang) {
                return Some(s);
            }
        }
        None
    }

    /// Formats a localized string template by interpolating arguments into a stack buffer.
    ///
    /// Supports sequential `{}` placeholders as well as positional `{0}`, `{1}` placeholders.
    pub fn translate_fmt<'b, const CAP: usize>(
        &self,
        key: &'a str,
        args: &[&str],
        buf: &'b mut heapless::String<CAP>,
    ) -> Result<&'b str, I18nError> {
        self.translate_fmt_lang(key, self.active_lang, args, buf)
    }

    /// Formats a localized string template for an explicit language by interpolating arguments into a stack buffer.
    pub fn translate_fmt_lang<'b, const CAP: usize>(
        &self,
        key: &'a str,
        lang: LanguageId,
        args: &[&str],
        buf: &'b mut heapless::String<CAP>,
    ) -> Result<&'b str, I18nError> {
        buf.clear();
        let template = self.translate_lang(key, lang);
        interpolate_template(template, args, buf)?;
        Ok(buf.as_str())
    }
}

/// Error conditions that can occur during translation and template formatting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum I18nError {
    /// The destination buffer capacity was exceeded during string formatting.
    BufferOverflow,
    /// A formatting template placeholder syntax was invalid (e.g. unclosed `{` or non-numeric index).
    InvalidPlaceholder,
    /// An interpolated positional placeholder index was not provided in the arguments slice.
    MissingArgument,
}

/// Zero-allocation template interpolator supporting `{}` and `{0}`, `{1}` placeholders into a `heapless::String`.
pub fn interpolate_template<const CAP: usize>(
    template: &str,
    args: &[&str],
    buf: &mut heapless::String<CAP>,
) -> Result<(), I18nError> {
    let mut chars = template.chars().peekable();
    let mut sequential_idx = 0usize;

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                chars.next();
                buf.push('{').map_err(|_| I18nError::BufferOverflow)?;
                continue;
            }

            let mut num_val: Option<usize> = None;
            let mut is_closing = false;

            while let Some(&next_ch) = chars.peek() {
                if next_ch == '}' {
                    chars.next();
                    is_closing = true;
                    break;
                } else if next_ch.is_ascii_digit() {
                    chars.next();
                    let digit = (next_ch as u8 - b'0') as usize;
                    num_val = Some(num_val.unwrap_or(0) * 10 + digit);
                } else {
                    return Err(I18nError::InvalidPlaceholder);
                }
            }

            if !is_closing {
                return Err(I18nError::InvalidPlaceholder);
            }

            let arg_idx = match num_val {
                Some(idx) => idx,
                None => {
                    let idx = sequential_idx;
                    sequential_idx += 1;
                    idx
                }
            };

            if let Some(&arg) = args.get(arg_idx) {
                buf.push_str(arg).map_err(|_| I18nError::BufferOverflow)?;
            } else {
                return Err(I18nError::MissingArgument);
            }
        } else if ch == '}' {
            if chars.peek() == Some(&'}') {
                chars.next();
                buf.push('}').map_err(|_| I18nError::BufferOverflow)?;
            } else {
                return Err(I18nError::InvalidPlaceholder);
            }
        } else {
            buf.push(ch).map_err(|_| I18nError::BufferOverflow)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static ENTRIES: [TranslationEntry<'static>; 4] = [
        TranslationEntry::new("btn_start", &["Start", "Iniciar", "Starten"]),
        TranslationEntry::new("btn_stop", &["Stop", "Detener", "Stoppen"]),
        TranslationEntry::new(
            "status_fmt",
            &["Status: {} ({1})", "Estado: {} ({1})", "Status: {} ({1})"],
        ),
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

    #[test]
    fn test_sorted_binary_search_lookup() {
        let sorted_tr = TranslationTable::new_sorted(&ENTRIES);
        assert!(sorted_tr.is_sorted);
        assert_eq!(sorted_tr.translate("btn_start"), "Start");
        assert_eq!(sorted_tr.translate("status_ready"), "Ready");
        assert_eq!(sorted_tr.translate("unknown"), "unknown");
    }

    #[test]
    fn test_message_id_indexed_lookup() {
        let tr = TranslationTable::new(&ENTRIES);
        assert_eq!(tr.translate_id(MessageId(0)), Some("Start"));
        assert_eq!(tr.translate_id(MessageId(1)), Some("Stop"));
        assert_eq!(tr.translate_id(MessageId(3)), Some("Ready"));
        assert_eq!(tr.translate_id(MessageId(99)), None);

        assert_eq!(
            tr.translate_id_lang(MessageId(1), LanguageId::ES),
            Some("Detener")
        );
    }

    #[test]
    fn test_template_interpolation() {
        let tr = TranslationTable::new(&ENTRIES);
        let mut buf: heapless::String<64> = heapless::String::new();

        let res = tr
            .translate_fmt("status_fmt", &["Online", "99%"], &mut buf)
            .unwrap();
        assert_eq!(res, "Status: Online (99%)");

        let mut es_tr = tr;
        es_tr.set_language(LanguageId::ES);
        let res_es = es_tr
            .translate_fmt("status_fmt", &["En línea", "99%"], &mut buf)
            .unwrap();
        assert_eq!(res_es, "Estado: En línea (99%)");
    }
}
