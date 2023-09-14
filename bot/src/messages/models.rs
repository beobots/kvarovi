use anyhow::anyhow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(sqlx::Type, Debug, Clone, Copy)]
#[sqlx(type_name = "message_type", rename_all = "lowercase")]
pub enum MessageType {
    Text,
    Command,
}

impl AsRef<str> for MessageType {
    fn as_ref(&self) -> &str {
        match self {
            MessageType::Text => "text",
            MessageType::Command => "command",
        }
    }
}

impl Display for MessageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl FromStr for MessageType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "text" => Ok(MessageType::Text),
            "command" => Ok(MessageType::Command),
            _ => Err(anyhow!("unknown message type [{s}]")),
        }
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct Message {
    pub chat_id: i64,
    pub text: String,
    #[sqlx(rename = "type")]
    pub message_type: MessageType,
}
