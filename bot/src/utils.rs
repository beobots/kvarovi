use crate::preferences::Language;
use rust_i18n::t as _t;

pub trait Escape {
    fn escape_markdown(&self) -> String;
}

pub fn t(key: &str, locale: Language) -> String {
    _t!(key, locale = locale.as_ref()).escape_markdown()
}

pub fn escape_markdown(text: &str) -> String {
    text.replace('_', "\\_")
        .replace('*', "\\*")
        .replace('[', "\\[")
        .replace('`', "\\`")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('~', "\\~")
        .replace('-', "\\-")
        .replace('.', "\\.")
        .replace('!', "\\!")
}

impl<T> Escape for T
where
    T: AsRef<str>,
{
    fn escape_markdown(&self) -> String {
        escape_markdown(self.as_ref())
    }
}
