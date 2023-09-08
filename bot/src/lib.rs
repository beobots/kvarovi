pub mod message_handler;
pub mod repositories;
pub mod repository;
pub mod utils;

pub mod messages;
pub mod preferences;

#[cfg(test)]
pub mod test;

rust_i18n::i18n!("locales", fallback = "en");
