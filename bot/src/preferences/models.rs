use anyhow::anyhow;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, sqlx::Type)]
pub enum Language {
    En,
    Ru,
    Rs,
}

impl AsRef<str> for Language {
    fn as_ref(&self) -> &str {
        match self {
            Language::En => "en",
            Language::Ru => "ru",
            Language::Rs => "rs",
        }
    }
}

impl FromStr for Language {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en" => Ok(Language::En),
            "ru" => Ok(Language::Ru),
            "rs" => Ok(Language::Rs),
            lang => Err(anyhow!("unknown language {lang}")),
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct ChatPreference {
    pub id: i32,
    pub chat_id: i64,
    pub language: Language,
}

pub struct NewChatPreference {
    pub chat_id: i64,
    pub language: Language,
}
