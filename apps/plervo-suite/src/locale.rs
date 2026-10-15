use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[default]
    Auto,
    En,
    Pt,
    Es,
}
impl Language {
    pub fn resolve(self) -> Self {
        if self != Self::Auto {
            return self;
        }
        from_tags(sys_locale::get_locales().map(|s| s.to_string()))
    }
    pub fn text<'a>(self, en: &'a str, pt: &'a str, es: &'a str) -> &'a str {
        match self {
            Self::Pt => pt,
            Self::Es => es,
            _ => en,
        }
    }
}
pub fn from_tags(tags: impl IntoIterator<Item = String>) -> Language {
    for tag in tags {
        match tag.split(['-', '_', '.']).next().unwrap_or_default().to_ascii_lowercase().as_str() {
            "en" => return Language::En,
            "pt" => return Language::Pt,
            "es" => return Language::Es,
            _ => {}
        }
    }
    Language::En
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn follows_os_preference_order_without_assuming_portuguese() {
        assert_eq!(Language::default(), Language::Auto);
        for (tags, want) in [
            (vec!["en-US", "pt-BR"], Language::En),
            (vec!["pt_BR.UTF-8", "en"], Language::Pt),
            (vec!["es-MX"], Language::Es),
            (vec!["fr", "es-ES"], Language::Es),
            (vec![], Language::En),
        ] {
            assert_eq!(from_tags(tags.into_iter().map(str::to_string)), want);
        }
    }
}
