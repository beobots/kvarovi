use std::sync::RwLock;

use crate::preferences::{ChatPreference, Language, Repository};
use anyhow::Result;
use async_trait::async_trait;

pub struct TestChatPreference {
    pub chat_preferences: RwLock<Vec<ChatPreference>>,
}

impl TestChatPreference {
    pub fn new() -> Self {
        Self {
            chat_preferences: RwLock::new(vec![]),
        }
    }

    pub fn chat_preferences(&self) -> Vec<ChatPreference> {
        self.chat_preferences.read().unwrap().to_vec()
    }

    pub fn set_chat_preferences(&self, chat_preferences: Vec<ChatPreference>) {
        let mut writer = self.chat_preferences.write().unwrap();

        writer.clear();
        writer.extend(chat_preferences);
    }
}

#[async_trait]
impl Repository for TestChatPreference {
    async fn insert(&self, value: ChatPreference) -> Result<()> {
        let new_chat_preference = ChatPreference {
            chat_id: value.chat_id,
            language: value.language,
        };

        let mut chat_preferences = self.chat_preferences.write().unwrap();

        chat_preferences.push(new_chat_preference);

        Ok(())
    }

    async fn find_one_by_chat_id(&self, chat_id: i64) -> Result<Option<ChatPreference>> {
        let chat_preferences = self.chat_preferences();

        let chat_preference = chat_preferences
            .iter()
            .find(|chat_preference| chat_preference.chat_id == chat_id)
            .and_then(|chat_preference| Some(chat_preference.to_owned()));

        Ok(chat_preference)
    }

    async fn update_language(&self, chat_id: i64, language: Language) -> Result<()> {
        let chat_preferences = self.chat_preferences();

        let modified_preferences = chat_preferences
            .iter()
            .map(|chat_preference| {
                if chat_preference.chat_id == chat_id {
                    ChatPreference {
                        chat_id: chat_preference.chat_id,
                        language,
                    }
                } else {
                    chat_preference.clone()
                }
            })
            .collect::<Vec<ChatPreference>>();

        self.set_chat_preferences(modified_preferences);

        Ok(())
    }
}
