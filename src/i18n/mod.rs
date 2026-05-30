pub mod en;
pub mod zh;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Language {
    #[serde(rename = "en")]
    #[default]
    En,
    #[serde(rename = "zh")]
    Zh,
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "en" => Ok(Language::En),
            "zh" => Ok(Language::Zh),
            _ => Err(format!("Unsupported language: '{}'. Use 'en' or 'zh'", s)),
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::En => write!(f, "en"),
            Language::Zh => write!(f, "zh"),
        }
    }
}

/// Language configuration holder
#[derive(Debug, Clone, Default)]
pub struct LanguageConfig {
    pub current: Language,
}

lazy_static! {
    static ref TRANSLATIONS: HashMap<(&'static str, &'static str), &'static str> = {
        let mut map = HashMap::new();
        for (k, v) in en::translations() {
            map.insert(k, v);
        }
        for (k, v) in zh::translations() {
            map.insert(k, v);
        }
        map
    };
}

/// Look up a translation for the given language and key.
/// The key must be a string literal (i.e., `&'static str`).
/// Returns the key itself if the translation is not found.
pub fn tr(lang: Language, key: &'static str) -> &'static str {
    let lang_code = match lang {
        Language::En => "en",
        Language::Zh => "zh",
    };
    TRANSLATIONS.get(&(lang_code, key)).copied().unwrap_or(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_str() {
        assert_eq!(Language::from_str("en").unwrap(), Language::En);
        assert_eq!(Language::from_str("zh").unwrap(), Language::Zh);
        assert_eq!(Language::from_str("EN").unwrap(), Language::En);
        assert_eq!(Language::from_str("ZH").unwrap(), Language::Zh);
        assert!(Language::from_str("invalid").is_err());
        assert!(Language::from_str("fr").is_err());
    }

    #[test]
    fn test_language_display() {
        assert_eq!(format!("{}", Language::En), "en");
        assert_eq!(format!("{}", Language::Zh), "zh");
    }

    #[test]
    fn test_language_default() {
        let lang = Language::default();
        assert_eq!(lang, Language::En);
    }

    #[test]
    fn test_tr_english_cpu() {
        assert_eq!(tr(Language::En, "cpu"), "CPU");
    }

    #[test]
    fn test_tr_chinese_cpu() {
        assert_eq!(tr(Language::Zh, "cpu"), "CPU");
    }

    #[test]
    fn test_tr_unknown_key_fallback() {
        assert_eq!(
            tr(Language::En, "nonexistent_key_xyz"),
            "nonexistent_key_xyz"
        );
        assert_eq!(
            tr(Language::Zh, "nonexistent_key_xyz"),
            "nonexistent_key_xyz"
        );
    }

    #[test]
    fn test_all_en_keys_exist_in_zh() {
        let en_translations = en::translations();
        let zh_translations = zh::translations();
        let zh_keys: std::collections::HashSet<&str> =
            zh_translations.iter().map(|((_, key), _)| *key).collect();

        for ((_, key), _) in &en_translations {
            assert!(
                zh_keys.contains(key),
                "English key '{}' not found in Chinese translations",
                key
            );
        }
    }

    #[test]
    fn test_all_zh_keys_exist_in_en() {
        let en_translations = en::translations();
        let zh_translations = zh::translations();
        let en_keys: std::collections::HashSet<&str> =
            en_translations.iter().map(|((_, key), _)| *key).collect();

        for ((_, key), _) in &zh_translations {
            assert!(
                en_keys.contains(key),
                "Chinese key '{}' not found in English translations",
                key
            );
        }
    }

    #[test]
    fn test_language_serde_roundtrip() {
        // Serialize Language::En
        let json_en = serde_json::to_string(&Language::En).unwrap();
        assert_eq!(json_en, "\"en\"");
        let deserialized_en: Language = serde_json::from_str(&json_en).unwrap();
        assert_eq!(deserialized_en, Language::En);

        // Serialize Language::Zh
        let json_zh = serde_json::to_string(&Language::Zh).unwrap();
        assert_eq!(json_zh, "\"zh\"");
        let deserialized_zh: Language = serde_json::from_str(&json_zh).unwrap();
        assert_eq!(deserialized_zh, Language::Zh);
    }

    #[test]
    fn test_translations_not_empty() {
        let en_count = en::translations().len();
        let zh_count = zh::translations().len();
        assert!(
            en_count > 30,
            "English translations has {} entries, expected > 30",
            en_count
        );
        assert!(
            zh_count > 30,
            "Chinese translations has {} entries, expected > 30",
            zh_count
        );
        assert_eq!(
            en_count, zh_count,
            "English and Chinese translations should have the same count"
        );
    }
}
