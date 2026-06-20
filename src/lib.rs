//! `langpack` — простой загрузчик языковых файлов.
//!
//! Формат строки в файле:
//! ```text
//! hello=ru="Привет"
//! hello=en="Hello"
//! # это комментарий
//! ```
//!
//! Использование:
//! ```
//! use uu_langpack::LangPack;
//!
//! let content = r#"hello=en="Hello"
//! hello=ru="Привет""#;
//!
//! let pack = LangPack::load(content, "en");
//! assert_eq!(pack.get("hello", "ru"), Some("Привет"));
//! assert_eq!(pack.get_or_key("missing", "ru"), "missing");
//! ```

use std::collections::HashMap;

/// Ключ -> (язык -> значение)
pub type Translations = HashMap<String, HashMap<String, String>>;

/// Языковой пакет: данные переводов + язык по умолчанию для fallback.
#[derive(Debug, Clone)]
pub struct LangPack {
    data: Translations,
    default_lang: String,
}

impl LangPack {
    /// Загружает переводы из текстового содержимого файла.
    ///
    /// Строки вида `key=lang="value"`. Пустые строки и строки,
    /// начинающиеся с `#`, игнорируются. Некорректные строки
    /// (не соответствующие формату) молча пропускаются.
    pub fn load(content: &str, default_lang: impl Into<String>) -> Self {
        let mut data: Translations = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, lang, value)) = parse_line(line) {
                data.entry(key).or_default().insert(lang, value);
            }
        }

        Self {
            data,
            default_lang: default_lang.into(),
        }
    }

    /// Создаёт пустой пакет (без перевода) — полезно для тестов
    /// или как заглушка до загрузки реальных данных.
    pub fn empty(default_lang: impl Into<String>) -> Self {
        Self {
            data: Translations::new(),
            default_lang: default_lang.into(),
        }
    }

    /// Возвращает перевод `key` для `lang`, с fallback на `default_lang`.
    /// `None`, если ключа нет вовсе ни на запрошенном, ни на дефолтном языке.
    pub fn get(&self, key: &str, lang: &str) -> Option<&str> {
        let variants = self.data.get(key)?;
        variants
            .get(lang)
            .or_else(|| variants.get(&self.default_lang))
            .map(String::as_str)
    }

    /// Как [`LangPack::get`], но при полном отсутствии перевода
    /// возвращает сам ключ — удобно для шаблонов, где нужна
    /// гарантированная строка без `Option`.
    pub fn get_or_key<'a>(&'a self, key: &'a str, lang: &str) -> &'a str {
        self.get(key, lang).unwrap_or(key)
    }

    /// Список языков, для которых существует хотя бы один перевод.
    pub fn languages(&self) -> Vec<&str> {
        let mut set: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for variants in self.data.values() {
            set.extend(variants.keys().map(String::as_str));
        }
        let mut langs: Vec<&str> = set.into_iter().collect();
        langs.sort_unstable();
        langs
    }

    /// Язык, используемый как fallback.
    pub fn default_lang(&self) -> &str {
        &self.default_lang
    }

    /// Количество загруженных ключей.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// `true`, если переводов нет вовсе.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Возвращает все переводы для языка `lang` в виде плоской `HashMap`.
    ///
    /// Ключ результирующей мапы — идентификатор строки, значение — перевод.
    /// Если для конкретного ключа нет перевода на запрошенном языке,
    /// подставляется значение из [`default_lang`](Self::default_lang).
    /// Ключи без перевода ни на запрошенном, ни на дефолтном языке
    /// пропускаются (не попадают в результат).
    ///
    /// # Пример
    ///
    /// ```
    /// use uu_langpack::LangPack;
    ///
    /// let content = r#"
    /// hello=ru="Привет"
    /// hello=en="Hello"
    /// bye=ru="Пока"
    /// "#;
    ///
    /// let pack = LangPack::load(content, "en");
    ///
    /// let ru = pack.to_hashmap_by_lang("ru");
    /// assert_eq!(ru.get("hello"), Some(&"Привет".to_string()));
    /// assert_eq!(ru.get("bye"), Some(&"Пока".to_string()));
    ///
    /// // "unknown_key" есть только на "en", поэтому fallback сработает
    /// let en = pack.to_hashmap_by_lang("en");
    /// assert_eq!(en.get("hello"), Some(&"Hello".to_string()));
    /// ```
    pub fn to_hashmap_by_lang(&self, lang: &str) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for (key, variants) in &self.data {
            if let Some(value) = variants
                .get(lang)
                .or_else(|| variants.get(&self.default_lang))
            {
                result.insert(key.clone(), value.clone());
            }
        }
        result
    }
}

/// Разбирает одну строку формата `key=lang="value"`.
///
/// Возвращает `None`, если строка не соответствует формату:
/// - ключ должен состоять из `[a-zA-Z0-9_-]` и быть непустым
/// - код языка должен состоять ровно из 2 букв
/// - значение должно быть в двойных кавычках (могут быть пустыми: `""`)
fn parse_line(line: &str) -> Option<(String, String, String)> {
    let mut parts = line.splitn(3, '=');
    let key = parts
        .next()
        .expect("splitn всегда возвращает хотя бы один элемент");
    let lang = parts.next()?;
    let quoted_value = parts.next()?;

    if !is_valid_key(key) {
        return None;
    }
    if !is_valid_lang(lang) {
        return None;
    }

    let value = quoted_value.strip_prefix('"')?.strip_suffix('"')?;

    Some((key.to_string(), lang.to_lowercase(), value.to_string()))
}

fn is_valid_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn is_valid_lang(lang: &str) -> bool {
    lang.len() == 2 && lang.chars().all(|c| c.is_ascii_alphabetic())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_line ---

    #[test]
    fn parse_line_basic() {
        let result = parse_line(r#"hello=en="Hello""#);
        assert_eq!(
            result,
            Some(("hello".to_string(), "en".to_string(), "Hello".to_string()))
        );
    }

    #[test]
    fn parse_line_lowercases_lang_code() {
        let result = parse_line(r#"hello=EN="Hello""#);
        assert_eq!(result.unwrap().1, "en");
    }

    #[test]
    fn parse_line_empty_value() {
        let line = "hello=en=\"\"";
        let result = parse_line(line);
        assert_eq!(
            result,
            Some(("hello".to_string(), "en".to_string(), String::new()))
        );
    }

    #[test]
    fn parse_line_value_contains_equals_sign() {
        // splitn(3, '=') должен оставить '=' внутри значения нетронутым
        let result = parse_line(r#"formula=en="a=b+c""#);
        assert_eq!(
            result,
            Some(("formula".to_string(), "en".to_string(), "a=b+c".to_string()))
        );
    }

    #[test]
    fn parse_line_key_with_underscore_and_dash() {
        let result = parse_line(r#"greeting_msg-1=en="Hi""#);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, "greeting_msg-1");
    }

    #[test]
    fn parse_line_rejects_empty_key() {
        assert_eq!(parse_line(r#"=en="Hello""#), None);
    }

    #[test]
    fn parse_line_rejects_invalid_key_chars() {
        assert_eq!(parse_line(r#"hello world=en="Hello""#), None);
    }

    #[test]
    fn parse_line_rejects_lang_too_short() {
        assert_eq!(parse_line(r#"hello=e="Hello""#), None);
    }

    #[test]
    fn parse_line_rejects_lang_too_long() {
        assert_eq!(parse_line(r#"hello=eng="Hello""#), None);
    }

    #[test]
    fn parse_line_rejects_lang_with_digits() {
        assert_eq!(parse_line(r#"hello=e1="Hello""#), None);
    }

    #[test]
    fn parse_line_rejects_missing_quotes() {
        assert_eq!(parse_line(r#"hello=en=Hello"#), None);
    }

    #[test]
    fn parse_line_rejects_missing_closing_quote() {
        assert_eq!(parse_line(r#"hello=en="Hello"#), None);
    }

    #[test]
    fn parse_line_rejects_too_few_parts() {
        assert_eq!(parse_line("hello=en"), None);
        assert_eq!(parse_line("hello"), None);
    }

    #[test]
    fn parse_line_rejects_empty_line() {
        // splitn("", 3, '=') даёт один элемент [""], не None —
        // упадёт на is_valid_key("") (пустой ключ), не на самом ?.
        assert_eq!(parse_line(""), None);
    }

    #[test]
    fn parse_line_unicode_value() {
        let result = parse_line(r#"hello=ru="Привет, мир!""#);
        assert_eq!(result.unwrap().2, "Привет, мир!");
    }

    // --- LangPack::load ---

    #[test]
    fn load_basic_multilang() {
        let content = r#"
hello=en="Hello"
hello=ru="Привет"
bye=en="Bye"
"#;
        let pack = LangPack::load(content, "en");
        assert_eq!(pack.get("hello", "en"), Some("Hello"));
        assert_eq!(pack.get("hello", "ru"), Some("Привет"));
        assert_eq!(pack.get("bye", "en"), Some("Bye"));
    }

    #[test]
    fn load_ignores_comments_and_blank_lines() {
        let content = r#"
# это комментарий
hello=en="Hello"

# ещё комментарий
bye=en="Bye"
"#;
        let pack = LangPack::load(content, "en");
        assert_eq!(pack.len(), 2);
    }

    #[test]
    fn load_ignores_malformed_lines_silently() {
        let content = r#"
hello=en="Hello"
this is garbage
bye=en="Bye"
"#;
        let pack = LangPack::load(content, "en");
        assert_eq!(pack.len(), 2);
        assert_eq!(pack.get("hello", "en"), Some("Hello"));
    }

    #[test]
    fn load_trims_whitespace_around_lines() {
        let content = "   hello=en=\"Hello\"   \n";
        let pack = LangPack::load(content, "en");
        assert_eq!(pack.get("hello", "en"), Some("Hello"));
    }

    #[test]
    fn load_empty_content_gives_empty_pack() {
        let pack = LangPack::load("", "en");
        assert!(pack.is_empty());
    }

    // --- load: некорректные файлы целиком ---

    #[test]
    fn load_file_with_only_garbage_gives_empty_pack() {
        let content = "\
this is not valid
another bad line
12345
=====
";
        let pack = LangPack::load(content, "en");
        assert!(pack.is_empty());
    }

    #[test]
    fn load_file_with_only_comments_gives_empty_pack() {
        let content = "\
# заголовок файла
# автор: кто-то
# дата: 2026
";
        let pack = LangPack::load(content, "en");
        assert!(pack.is_empty());
    }

    #[test]
    fn load_file_with_only_blank_lines_gives_empty_pack() {
        let content = "\n\n   \n\t\n\n";
        let pack = LangPack::load(content, "en");
        assert!(pack.is_empty());
    }

    #[test]
    fn load_recovers_after_bad_line_mid_file() {
        // повреждённая строка не должна "сломать" парсинг последующих строк
        let content = "\
hello=en=\"Hello\"
broken line without proper format
bye=en=\"Bye\"
another=broken=line=with=too=many=equals=signs=\"value\"
thanks=en=\"Thanks\"
";
        let pack = LangPack::load(content, "en");
        assert_eq!(pack.get("hello", "en"), Some("Hello"));
        assert_eq!(pack.get("bye", "en"), Some("Bye"));
        assert_eq!(pack.get("thanks", "en"), Some("Thanks"));
    }

    #[test]
    fn load_rejects_line_with_unterminated_quote() {
        let content = "hello=en=\"Hello\nbye=en=\"Bye\"";
        // первая строка обрывается без закрывающей кавычки на той же строке
        // (перевод строки внутри кавычек не поддерживается формату),
        // вторая корректна и должна загрузиться
        let pack = LangPack::load(content, "en");
        assert_eq!(pack.get("bye", "en"), Some("Bye"));
        assert_eq!(pack.get("hello", "en"), None);
    }

    #[test]
    fn load_rejects_line_missing_equals_signs() {
        let content = "\
hello en Hello
hello:en:\"Hello\"
hello en=\"Hello\"
";
        let pack = LangPack::load(content, "en");
        assert!(pack.is_empty());
    }

    #[test]
    fn load_rejects_lang_code_with_three_letters() {
        // трёхбуквенные коды (ISO 639-2) не поддерживаются форматом
        let content = "hello=eng=\"Hello\"";
        let pack = LangPack::load(content, "en");
        assert!(pack.is_empty());
    }

    #[test]
    fn load_rejects_lang_code_with_region() {
        // en-US/en_US не должны проходить как двухбуквенный код
        let content = "\
hello=en-US=\"Hello\"
hi=en_US=\"Hi\"
";
        let pack = LangPack::load(content, "en");
        assert!(pack.is_empty());
    }

    #[test]
    fn load_rejects_key_with_spaces() {
        let content = "hello world=en=\"Hello\"";
        let pack = LangPack::load(content, "en");
        assert!(pack.is_empty());
    }

    #[test]
    fn load_rejects_key_with_dot_or_special_chars() {
        let content = "\
hello.world=en=\"Hello\"
hello@world=en=\"Hello\"
hello/world=en=\"Hello\"
";
        let pack = LangPack::load(content, "en");
        assert!(pack.is_empty());
    }

    #[test]
    fn load_rejects_value_without_quotes_at_all() {
        let content = "hello=en=Hello";
        let pack = LangPack::load(content, "en");
        assert!(pack.is_empty());
    }

    #[test]
    fn load_rejects_value_with_single_quotes_instead_of_double() {
        let content = "hello=en='Hello'";
        let pack = LangPack::load(content, "en");
        assert!(pack.is_empty());
    }

    #[test]
    fn load_handles_mixed_valid_and_structurally_broken_lines() {
        let content = "\
# комментарий в начале
valid_key=en=\"Valid\"

broken=key=with=too=many=parts
also broken
valid_key=ru=\"Валидно\"

# ещё комментарий
final_key=en=\"Final\"
";
        let pack = LangPack::load(content, "en");
        assert_eq!(pack.len(), 2);
        assert_eq!(pack.get("valid_key", "en"), Some("Valid"));
        assert_eq!(pack.get("valid_key", "ru"), Some("Валидно"));
        assert_eq!(pack.get("final_key", "en"), Some("Final"));
    }

    #[test]
    fn load_rejects_line_with_only_equals_signs() {
        let content = "===";
        let pack = LangPack::load(content, "en");
        assert!(pack.is_empty());
    }

    #[test]
    fn load_rejects_line_starting_with_equals() {
        let content = "=en=\"Hello\"";
        let pack = LangPack::load(content, "en");
        assert!(pack.is_empty());
    }

    #[test]
    fn load_ignores_byte_order_mark_line_as_garbage() {
        // BOM в начале файла иногда попадает как часть первой строки;
        // такая строка не должна паниковать, просто не распарсится
        let content = "\u{feff}hello=en=\"Hello\"";
        let pack = LangPack::load(content, "en");
        // первая строка испорчена BOM-префиксом и не пройдёт is_valid_key,
        // т.к. символ BOM не входит в разрешённый набор символов ключа
        assert!(pack.is_empty());
    }

    #[test]
    fn load_does_not_panic_on_binary_garbage() {
        // содержимое с null-байтами и непечатными символами не должно
        // приводить к панике парсера, только к игнорированию строк
        let content = "\u{0}\u{1}\u{2}\nhello=en=\"Hello\"\n\u{7f}";
        let pack = LangPack::load(content, "en");
        assert_eq!(pack.get("hello", "en"), Some("Hello"));
        assert_eq!(pack.len(), 1);
    }

    #[test]
    fn load_rejects_duplicate_equals_in_lang_position() {
        let content = "hello===\"Hello\"";
        let pack = LangPack::load(content, "en");
        assert!(pack.is_empty());
    }

    // --- get / get_or_key (fallback logic) ---

    #[test]
    fn get_returns_requested_lang_when_present() {
        let content = r#"hello=en="Hello"
hello=ru="Привет""#;
        let pack = LangPack::load(content, "en");
        assert_eq!(pack.get("hello", "ru"), Some("Привет"));
    }

    #[test]
    fn get_falls_back_to_default_lang_when_missing() {
        let content = r#"hello=en="Hello""#;
        let pack = LangPack::load(content, "en");
        // запрошенного "ru" нет, должны получить fallback "en"
        assert_eq!(pack.get("hello", "ru"), Some("Hello"));
    }

    #[test]
    fn get_returns_none_when_key_missing_entirely() {
        let pack = LangPack::load(r#"hello=en="Hello""#, "en");
        assert_eq!(pack.get("missing_key", "en"), None);
    }

    #[test]
    fn get_returns_none_when_key_exists_but_no_lang_and_no_default() {
        // ключ есть, но ни запрошенного, ни дефолтного языка нет
        let content = r#"hello=fr="Bonjour""#;
        let pack = LangPack::load(content, "en");
        assert_eq!(pack.get("hello", "ru"), None);
    }

    #[test]
    fn get_or_key_returns_key_when_nothing_found() {
        let pack = LangPack::load(r#"hello=en="Hello""#, "en");
        assert_eq!(pack.get_or_key("missing_key", "ru"), "missing_key");
    }

    #[test]
    fn get_or_key_returns_value_when_found() {
        let pack = LangPack::load(r#"hello=en="Hello""#, "en");
        assert_eq!(pack.get_or_key("hello", "en"), "Hello");
    }

    // --- languages() ---

    #[test]
    fn languages_lists_all_distinct_langs_sorted() {
        let content = r#"
hello=en="Hello"
hello=ru="Привет"
bye=fr="Au revoir"
"#;
        let pack = LangPack::load(content, "en");
        assert_eq!(pack.languages(), vec!["en", "fr", "ru"]);
    }

    #[test]
    fn languages_empty_for_empty_pack() {
        let pack = LangPack::empty("en");
        assert!(pack.languages().is_empty());
    }

    // --- misc ---

    #[test]
    fn default_lang_accessor() {
        let pack = LangPack::empty("ru");
        assert_eq!(pack.default_lang(), "ru");
    }

    #[test]
    fn len_and_is_empty() {
        let pack = LangPack::load(r#"hello=en="Hello""#, "en");
        assert_eq!(pack.len(), 1);
        assert!(!pack.is_empty());

        let empty = LangPack::empty("en");
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn duplicate_key_lang_pair_last_one_wins() {
        let content = r#"
hello=en="First"
hello=en="Second"
"#;
        let pack = LangPack::load(content, "en");
        assert_eq!(pack.get("hello", "en"), Some("Second"));
    }
}
