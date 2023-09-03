use chrono::Utc;
use teloxide_core::types::{
    Chat, ChatId, ChatKind, ChatPublic, MediaKind, MediaPhoto, Message, MessageCommon, MessageId, MessageKind,
    PublicChatGroup, PublicChatKind, Update, UpdateKind, User, UserId,
};

use crate::preferences::Language;

fn get_default_chat_public() -> ChatPublic {
    ChatPublic {
        title: None,
        kind: PublicChatKind::Group(PublicChatGroup { permissions: None }),
        description: None,
        invite_link: None,
        has_protected_content: None,
    }
}

fn get_default_user() -> User {
    User {
        id: UserId(1),
        is_bot: false,
        first_name: String::from("John"),
        last_name: Some(String::from("Doe")),
        username: Some(String::from("johndoe")),
        language_code: Some(String::from("en")),
        is_premium: false,
        added_to_attachment_menu: false,
    }
}

fn get_default_common_message() -> MessageCommon {
    MessageCommon {
        from: Some(get_default_user()),
        sender_chat: None,
        author_signature: None,
        forward: None,
        reply_to_message: None,
        edit_date: None,
        media_kind: MediaKind::Photo(MediaPhoto {
            photo: vec![],
            caption: None,
            caption_entities: vec![],
            has_media_spoiler: false,
            media_group_id: None,
        }),
        reply_markup: None,
        is_topic_message: false,
        is_automatic_forward: false,
        has_protected_content: false,
    }
}

fn get_default_message() -> Message {
    Message {
        id: MessageId(1),
        thread_id: None,
        date: Utc::now(),
        via_bot: None,
        chat: Chat {
            id: ChatId(5),
            kind: ChatKind::Public(get_default_chat_public()),
            photo: None,
            pinned_message: None,
            message_auto_delete_time: None,
            has_hidden_members: false,
            has_aggressive_anti_spam_enabled: false,
        },
        kind: MessageKind::Common(get_default_common_message()),
    }
}

pub struct BotUpdateMock {
    update: Update,
}

impl BotUpdateMock {
    pub fn new() -> Self {
        Self {
            update: Update {
                id: 1,
                kind: UpdateKind::Message(get_default_message()),
            },
        }
    }

    pub fn get_update(&self) -> Update {
        self.update.clone()
    }

    pub fn set_user_language(&mut self, language: Language) {
        let message = match &mut self.update.kind {
            UpdateKind::Message(message) => message,
            _ => panic!("Update is not a message"),
        };

        let common_message: &mut MessageCommon = match &mut message.kind {
            MessageKind::Common(message) => message,
            _ => panic!("Message is not common"),
        };

        let user = match &mut common_message.from {
            Some(user) => user,
            None => panic!("Message has no user"),
        };

        user.language_code = Some(language.as_ref().to_owned());
    }
}
