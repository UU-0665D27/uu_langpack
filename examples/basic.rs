//! Базовое использование `LangPack`.
//!
//! Запуск: `cargo run --example basic`

use uu_langpack::LangPack;

fn main() {
    let content = r#"
# Главная страница
hello=en="Hello"
hello=ru="Привет"
bye=en="Bye"
bye=ru="Пока"
"#;

    let pack = LangPack::load(content, "en");

    println!("Доступные языки: {:?}", pack.languages());
    println!("Всего ключей: {}", pack.len());

    println!();
    println!("hello/ru -> {:?}", pack.get("hello", "ru"));
    println!("hello/en -> {:?}", pack.get("hello", "en"));

    // Ключа нет вовсе — get вернёт None
    println!("missing/ru -> {:?}", pack.get("missing", "ru"));

    // get_or_key никогда не возвращает None — fallback на сам ключ
    println!(
        "missing/ru (or_key) -> {}",
        pack.get_or_key("missing", "ru")
    );
}
