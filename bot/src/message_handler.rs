use crate::preferences::{self, *};
use crate::repositories::message_repository::MessageType;
use crate::utils::{escape_markdown, t};
use crate::{
    repositories::{
        message_repository::{MessageRepository, NewMessage, Repository as _},
        subscription_repository::{NewSubscription, Repository as _, SubscriptionsRepository},
    },
    utils::Escape,
};
use anyhow::Context as _;
use anyhow::{Ok, Result};
use electricity::translit::Translit;
use rust_i18n::t as _t;
use sqlx::postgres::PgPoolOptions;
use std::str::FromStr;
use teloxide_core::{
    prelude::*,
    types::{
        ChatId, InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup,
        ParseMode, Update, UpdateKind,
    },
};
use tracing::debug;

fn get_settings_action_text(preference: &ChatPreference) -> String {
    format!("⚙️ {}", t("menu.settings", preference.language))
}

fn get_my_addresses_action_text(preference: &ChatPreference) -> String {
    format!("📝 {}", t("menu.my_address", preference.language))
}

fn get_subscribe_action_text(preference: &ChatPreference) -> String {
    format!("🔔 {}", t("menu.subscribe", preference.language))
}

fn get_check_address_action_text(preference: &ChatPreference) -> String {
    format!("📝 {}", t("menu.check_address", preference.language))
}

fn get_unsubscribe_action_text(preference: &ChatPreference) -> String {
    format!("🔕 {}", t("menu.unsubscribe", preference.language))
}

fn get_full_menu(preference: &ChatPreference) -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![KeyboardButton::new(get_check_address_action_text(
            preference,
        ))],
        vec![
            KeyboardButton::new(get_subscribe_action_text(preference)),
            KeyboardButton::new(get_unsubscribe_action_text(preference)),
        ],
        vec![
            KeyboardButton::new(get_my_addresses_action_text(preference)),
            KeyboardButton::new(get_settings_action_text(preference)),
        ],
    ])
    .resize_keyboard(true)
}

fn get_settings_actions(preference: &ChatPreference) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::new(
            format!("🌎 {}", t("settings.language", preference.language)),
            InlineKeyboardButtonKind::CallbackData("change_language".to_string()),
        ),
        InlineKeyboardButton::new(
            format!("🔔 {}", t("settings.notification", preference.language)),
            InlineKeyboardButtonKind::CallbackData("notification_settings".to_string()),
        ),
    ]])
}

fn get_language_actions(preference: &ChatPreference, with_back_button: Option<bool>) -> InlineKeyboardMarkup {
    let english = if preference.language == Language::En {
        "✅ 🇺🇸 English"
    } else {
        "🇺🇸 English"
    };
    let russian = if preference.language == Language::Ru {
        "✅ 🇷🇺 Русский"
    } else {
        "🇷🇺 Русский"
    };
    let serbian = if preference.language == Language::Rs {
        "✅ 🇷🇸 Српски"
    } else {
        "🇷🇸 Српски"
    };

    let mut languages = vec![
        vec![
            InlineKeyboardButton::new(
                english,
                InlineKeyboardButtonKind::CallbackData("change_language_en".to_string()),
            ),
            InlineKeyboardButton::new(
                russian,
                InlineKeyboardButtonKind::CallbackData("change_language_ru".to_string()),
            ),
        ],
        vec![InlineKeyboardButton::new(
            serbian,
            InlineKeyboardButtonKind::CallbackData("change_language_rs".to_string()),
        )],
    ];

    if Some(true) == with_back_button {
        languages.push(vec![InlineKeyboardButton::new(
            format!("🔙 {}", t("settings.back", preference.language)),
            InlineKeyboardButtonKind::CallbackData("languages_back".to_string()),
        )])
    }

    InlineKeyboardMarkup::new(languages)
}

async fn get_sqlx_database_client() -> Result<sqlx::PgPool> {
    let database_url = dotenvy::var("POSTGRESQL_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

fn get_update_language_code(update: &Update) -> Language {
    let mut language_code = Language::En;

    if let UpdateKind::Message(message) = &update.kind {
        if let Some(user) = &message.from() {
            if let Some(language_code_option) = &user.language_code {
                language_code = Language::from_str(language_code_option).unwrap_or(Language::En);
            }
        }
    }

    language_code
}

async fn get_chat_preference<T>(
    chat_preference_repository: &mut T,
    update: &Update,
    chat_id: i64,
) -> Result<ChatPreference>
where
    T: preferences::Repository,
{
    let chat_preference = chat_preference_repository.find_one(chat_id).await?;

    if let Some(chat_preference) = chat_preference {
        Ok(chat_preference)
    } else {
        let language_code = get_update_language_code(update);

        let new_chat_preference = ChatPreference {
            chat_id,
            language: language_code,
        };
        chat_preference_repository
            .insert(new_chat_preference)
            .await?;

        let chat_preference = chat_preference_repository.find_one(chat_id).await?;

        if let Some(chat_preferences) = chat_preference {
            Ok(chat_preferences)
        } else {
            Err(anyhow::anyhow!(
                "Failed to create chat preference for chat_id: {}",
                chat_id
            ))
        }
    }
}

pub async fn handle_update<T>(
    update: &Update,
    subscriptions_repository: SubscriptionsRepository<'_>,
    message_repository: MessageRepository<'_>,
    chat_preference_repository: &mut T,
) -> Result<()>
where
    T: preferences::Repository,
{
    let bot = Bot::from_env().parse_mode(ParseMode::MarkdownV2);

    subscriptions_repository.create_table().await?;
    message_repository.create_table().await?;

    debug!("Got update: {update:?}");

    match &update.kind {
        UpdateKind::Message(message) => {
            let chat_id = message.chat.id;
            let ChatId(chat_id_i64) = chat_id;
            let chat_preference = get_chat_preference(chat_preference_repository, update, chat_id_i64).await?;

            if let Some(text) = message.text() {
                let mut message_type = MessageType::Text;

                if text == "/start" {
                    message_type = MessageType::Command;
                    bot.send_message(chat_id, t("start", chat_preference.language))
                        .reply_markup(get_full_menu(&chat_preference))
                        .await?;
                } else if text == get_check_address_action_text(&chat_preference) {
                    message_type = MessageType::Command;
                    bot.send_message(chat_id, t("check_address_text", chat_preference.language))
                        .await?;
                } else if text == get_subscribe_action_text(&chat_preference) {
                    message_type = MessageType::Command;
                    bot.send_message(chat_id, "Введите ваш адрес").await?;
                } else if text == get_unsubscribe_action_text(&chat_preference) {
                    let subscriptions = subscriptions_repository
                        .find_all_by_chat_id(chat_id_i64)
                        .await?;

                    if !subscriptions.is_empty() {
                        let mut addresses = String::new();

                        for (index, subscription) in subscriptions.iter().enumerate() {
                            let address = subscription.address.to_owned();
                            addresses.push_str(&format!("\\[{}\\] {}\n", index, address));
                        }

                        message_type = MessageType::Command;
                        bot.send_message(
                            chat_id,
                            format!("Выберите адреса, которые хотите отписать:\n{}", addresses),
                        )
                        .await?;
                    } else {
                        bot.send_message(chat_id, "У вас нет подписок").await?;
                    }
                } else if text == get_my_addresses_action_text(&chat_preference) {
                    message_type = MessageType::Command;

                    let subscriptions = subscriptions_repository
                        .find_all_by_chat_id(chat_id_i64)
                        .await?;

                    if !subscriptions.is_empty() {
                        let mut addresses = String::new();

                        for (index, subscription) in subscriptions.iter().enumerate() {
                            let address = subscription.address.to_owned();
                            addresses.push_str(&format!("\\[{}\\] {}\n", index, address));
                        }

                        bot.send_message(chat_id, format!("Ваши адреса:\n{}", addresses))
                            .await?;
                    } else {
                        bot.send_message(chat_id, "У вас нет подписок. Хотите ли добавить?")
                            .await?;
                    }
                } else if text == get_settings_action_text(&chat_preference) {
                    bot.send_message(chat_id, t("settings.text", chat_preference.language))
                        .reply_markup(get_settings_actions(&chat_preference))
                        .await?;
                } else {
                    let last_command = message_repository
                        .find_one_by_chat_id(chat_id_i64, "command".to_owned())
                        .await?;

                    if let Some(last_command) = last_command {
                        let last_command_text = last_command.text.to_owned();
                        if last_command_text == get_check_address_action_text(&chat_preference) {
                            // Сделать что то с адресом
                            bot.send_message(chat_id, "Вот информация по вашему адресу")
                                .await?;
                        } else if last_command_text == get_subscribe_action_text(&chat_preference) {
                            subscriptions_repository
                                .insert(NewSubscription {
                                    chat_id: chat_id_i64,
                                    address: text.translit().to_owned(),
                                })
                                .await?;

                            bot.send_message(chat_id, t("subscribed", chat_preference.language))
                                .await?;
                        } else if last_command_text == get_unsubscribe_action_text(&chat_preference) {
                            let subscriptions = subscriptions_repository
                                .find_all_by_chat_id(chat_id_i64)
                                .await?;

                            let ids = text
                                .to_owned()
                                .replace(' ', ",")
                                .split(',')
                                .filter(|s| !s.is_empty())
                                .map(|s| s.parse::<usize>())
                                .filter_map(|s| s.ok())
                                .collect::<Vec<usize>>();

                            let ids_to_remove = ids
                                .iter()
                                .map(|index| subscriptions[index.to_owned()].id)
                                .map(i64::from)
                                .collect::<Vec<i64>>();

                            subscriptions_repository
                                .delete_by_ids(ids_to_remove)
                                .await?;

                            let removed_addresses = ids
                                .iter()
                                .map(|index| subscriptions[index.to_owned()].address.to_owned())
                                .collect::<Vec<String>>()
                                .join(",");

                            bot.send_message(
                                chat_id,
                                format!(
                                    "Вы отписались от уведомлений по адресам: {}",
                                    removed_addresses
                                ),
                            )
                            .await?;
                        } else {
                            bot.send_message(chat_id, "Неизвестная команда").await?;
                        }
                    }
                }

                debug!("Got message: {text:?}");

                message_repository
                    .insert(NewMessage {
                        chat_id: chat_id_i64,
                        text: text.to_owned(),
                        message_type: message_type.as_ref().to_owned(),
                    })
                    .await?;
            }

            if let Some(location) = message.location() {
                let latitude = location.latitude;
                let longitude = location.longitude;

                let last_command = message_repository
                    .find_one_by_chat_id(chat_id_i64, "command".to_owned())
                    .await?;

                if let Some(last_command) = last_command {
                    let last_command_text = last_command.text.to_owned();

                    if last_command_text == get_check_address_action_text(&chat_preference) {
                        // Сделать что то с адресом
                        bot.send_message(
                            chat_id,
                            escape_markdown(&format!(
                                "Вот информация по вашей геолокации: latitude: {}, longitude: {}",
                                latitude, longitude
                            )),
                        )
                        .await?;
                    }
                }
            }
        }
        UpdateKind::CallbackQuery(query) => {
            if let Some(message) = &query.message {
                let chat_id = message.chat.id;
                let ChatId(chat_id_i64) = chat_id;
                let mut chat_preference = get_chat_preference(chat_preference_repository, update, chat_id_i64).await?;

                if let Some(data) = &query.data {
                    if data == "change_language" {
                        bot.edit_message_text(
                            chat_id,
                            message.id,
                            t("settings.change_language_text", chat_preference.language),
                        )
                        .reply_markup(get_language_actions(&chat_preference, Some(true)))
                        .await?;
                    } else if data.starts_with("change_language_") {
                        let new_language = Language::from_str(&data.replace("change_language_", ""))
                            .with_context(|| format!("invalid chat language command: {data}"))?;

                        if new_language == chat_preference.language {
                            return Ok(());
                        }

                        chat_preference_repository
                            .update_language(chat_id_i64, new_language)
                            .await?;
                        chat_preference = get_chat_preference(chat_preference_repository, update, chat_id_i64).await?;

                        bot.edit_message_text(
                            chat_id,
                            message.id,
                            t("settings.change_language_text", chat_preference.language),
                        )
                        .reply_markup(get_language_actions(&chat_preference, None))
                        .await?;

                        bot.send_message(chat_id, t("settings.language_changed", new_language))
                            .reply_markup(get_full_menu(&chat_preference))
                            .await?;
                    } else if data == "notification_settings" {
                        bot.send_message(chat_id, "Not implemented yet").await?;
                    } else if data == "languages_back" {
                        bot.edit_message_text(
                            chat_id,
                            message.id,
                            t("settings.text", chat_preference.language),
                        )
                        .reply_markup(get_settings_actions(&chat_preference))
                        .await?;
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}

#[allow(dead_code)]
async fn send_message(chat_id: i64, message: &str) -> Result<()> {
    let bot = Bot::from_env().parse_mode(ParseMode::MarkdownV2);

    bot.send_message(ChatId(chat_id), escape_markdown(message))
        .await?;

    Ok(())
}

pub async fn notify_addresses(addresses: Vec<String>) -> Result<()> {
    let sqlx_database_client = get_sqlx_database_client().await?;

    let subscription_repository = SubscriptionsRepository::new(&sqlx_database_client);

    let subscriptions = subscription_repository
        .find_all_by_addresses(addresses)
        .await?;

    for subscription in subscriptions {
        let chat_id = subscription.chat_id;
        let chat_preference_repository = PgChatPreference::new(&sqlx_database_client);
        let chat_preference = chat_preference_repository.find_one(chat_id).await?;

        if let Some(chat_preference) = chat_preference {
            send_message(
                chat_id,
                _t!(
                    "shutdown_warning",
                    locale = chat_preference.language.as_ref(),
                    address = subscription.address
                )
                .escape_markdown()
                .as_str(),
            )
            .await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;

    #[tokio::test]
    async fn test_get_existing_chat_preference() {
        let mut repository = test::preferences::TestChatPreference::new();

        repository.set_chat_preferences(vec![
            ChatPreference {
                chat_id: 4,
                language: Language::En,
            },
            ChatPreference {
                chat_id: 5,
                language: Language::Ru,
            },
        ]);

        let update_mock = test::mock::BotUpdateMock::new();
        let update = update_mock.get_update();

        let found_chat_preference = get_chat_preference(&mut repository, &update, 5)
            .await
            .unwrap();

        assert_eq!(found_chat_preference.chat_id, 5);
        assert_eq!(found_chat_preference.language, Language::Ru);
    }

    #[tokio::test]
    async fn test_get_created_chat_preference() {
        let mut repository = test::preferences::TestChatPreference::new();

        let mut update_mock = test::mock::BotUpdateMock::new();
        update_mock.set_user_language(Language::Rs);
        let update = update_mock.get_update();

        let found_chat_preference = get_chat_preference(&mut repository, &update, 5)
            .await
            .unwrap();

        assert_eq!(found_chat_preference.chat_id, 5);
        assert_eq!(found_chat_preference.language, Language::Rs);
    }
}
