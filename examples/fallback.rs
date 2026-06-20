//! Демонстрация fallback-логики: если для запрошенного языка
//! перевода нет, используется `default_lang`, заданный при `load`.
//!
//! Запуск: `cargo run --example fallback`

use uu_langpack::LangPack;

fn main() {
    // у ключа "only_en" перевод есть только на английском
    let content = r#"
greeting=en="Welcome"
greeting=ru="Добро пожаловать"
only_en=en="English only string"
"#;

    let pack = LangPack::load(content, "en");

    // запрошенный язык есть -> возвращается он
    println!("greeting/ru -> {:?}", pack.get("greeting", "ru"));

    // запрошенного языка нет -> fallback на default_lang ("en")
    println!("only_en/ru -> {:?}", pack.get("only_en", "ru"));
    println!("only_en/fr -> {:?}", pack.get("only_en", "fr"));

    // ключа нет вообще ни на одном языке -> None
    println!("nonexistent/en -> {:?}", pack.get("nonexistent", "en"));

    // пакет с другим default_lang
    let ru_default_pack = LangPack::load(content, "ru");
    println!();
    println!("default_lang = ru");
    println!("only_en/fr -> {:?}", ru_default_pack.get("only_en", "fr"));
    // тут "only_en" нет даже на ru (default), поэтому None
}
