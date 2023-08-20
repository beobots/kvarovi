use rust_i18n::t as _t;

pub fn t(key: &str, locale: &str) -> String {
    escape_markdown(&_t!(key, locale = locale))
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
