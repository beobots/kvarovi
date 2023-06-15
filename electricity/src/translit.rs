//! To simplify text processing and models all the input text from users and
//! data obtained from web sites will be transliterated into Latin script and to
//! lower case register.
use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::OnceLock;

macro_rules! smap {
    ($map:ident, $to:expr, $($from:expr),+ $(,)? ) => {
        $(
            $map.insert($from, CharOrString::from($to));
        )+
    };
}

#[derive(Hash, PartialEq, Debug, Eq)]
enum CharOrString {
    Char(char),
    String(String),
}

impl From<char> for CharOrString {
    fn from(v: char) -> Self {
        CharOrString::Char(v)
    }
}

impl From<String> for CharOrString {
    fn from(v: String) -> Self {
        CharOrString::String(v)
    }
}

impl From<&'_ str> for CharOrString {
    fn from(v: &'_ str) -> Self {
        CharOrString::String(v.to_owned())
    }
}

static CHAR_MAP: OnceLock<HashMap<char, CharOrString>> = OnceLock::new();

pub trait Translit {
    fn translit(&self) -> String;
}

impl<T> Translit for T
where
    T: AsRef<str>,
{
    fn translit(&self) -> String {
        translit(self.as_ref())
    }
}

fn translit(input: &str) -> String {
    let map = CHAR_MAP.get_or_init(|| {
        let mut map = HashMap::new();

        smap![map, 'a', 'А', 'а'];
        smap![map, 'b', 'Б', 'б'];
        smap![map, 'c', 'Ц', 'ц'];
        smap![map, 'č', 'Ч', 'ч'];
        smap![map, 'ć', 'Ћ', 'ћ'];
        smap![map, 'd', 'Д', 'д'];
        smap![map, "dž", 'Џ', 'џ'];
        smap![map, 'đ', 'Ђ', 'ђ'];
        smap![map, 'e', 'Е', 'е'];
        smap![map, 'f', 'Ф', 'ф'];
        smap![map, 'g', 'Г', 'г'];
        smap![map, 'h', 'Х', 'х'];
        smap![map, 'i', 'И', 'и'];
        smap![map, 'j', 'Ј', 'ј'];
        smap![map, 'k', 'К', 'к'];
        smap![map, 'l', 'Л', 'л'];
        smap![map, "lj", 'Љ', 'љ'];
        smap![map, 'm', 'М', 'м'];
        smap![map, 'n', 'Н', 'н'];
        smap![map, "nj", 'Њ', 'њ'];
        smap![map, 'o', 'О', 'о'];
        smap![map, 'p', 'П', 'п'];
        smap![map, 'r', 'Р', 'р'];
        smap![map, 's', 'С', 'с'];
        smap![map, 'š', 'Ш', 'ш'];
        smap![map, 't', 'Т', 'т'];
        smap![map, 'u', 'У', 'у'];
        smap![map, 'v', 'В', 'в'];
        smap![map, 'z', 'З', 'з'];
        smap![map, 'ž', 'Ж', 'ж'];

        map
    });

    let string_iter = input.chars().map(|c| {
        if let Some(mapped_value) = map.get(&c) {
            match mapped_value {
                CharOrString::Char(rc) => rc.to_lowercase().to_string(),
                CharOrString::String(rs) => rs.to_lowercase(),
            }
        } else {
            c.to_lowercase().to_string()
        }
    });

    String::from_iter(string_iter)
}

#[cfg(test)]
mod tests {

    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_translit_empty_string() {
        let output = "".translit();
        assert_eq!(&output, "");
    }

    #[test]
    fn test_translit_cyrillic() {
        let output = "У служби грађана - Званична презентација Владе Републике Србије".translit();
        assert_eq!(
            &output,
            "u službi građana - zvanična prezentacija vlade republike srbije"
        );
    }

    proptest! {
        #[test]
        fn test_translit_to_lowercase(s in "\\PC*") {
            let result = s.translit();
            prop_assert_eq!(result.to_lowercase(), result);
        }

        #[test]
        fn test_translit_cyrillic_symb(s in "\\p{Cyrillic}{0,10}") {
            let result: String = s.translit();

            let map = CHAR_MAP.get().unwrap();
            prop_assert!(map.keys().all(|c| result.find(*c).is_none()));
        }
    }
}
