//! Fuzz-цель: `LangPack::load` на произвольных байтах.
//!
//! Цель — проверить, что парсер языкового файла никогда не паникует
//! и не выходит за границы памяти ни на каком вводе, включая
//! невалидный UTF-8, экстремально длинные строки, неэкранированные
//! кавычки, NUL-байты и т.п.
//!
//! Запуск:
//!   cargo fuzz run fuzz_load
//!
//! С санитайзерами (по умолчанию ASan включён cargo-fuzz):
//!   cargo fuzz run fuzz_load -- -max_len=65536

#![no_main]

use libfuzzer_sys::fuzz_target;
use uu_langpack::LangPack;

fuzz_target!(|data: &[u8]| {
    // Произвольные байты могут быть невалидным UTF-8 — это ожидаемо
    // и не должно приводить к панике, просто пропускаем такой ввод,
    // как сделал бы реальный код при чтении файла с диска.
    let Ok(content) = std::str::from_utf8(data) else {
        return;
    };

    let pack = LangPack::load(content, "en");

    // Опрашиваем пакет по ключам, которые реально могли встретиться
    // во входе (первое "слово" до '=' в каждой строке), плюс пара
    // синтетических ключей, которых точно не будет в данных —
    // чтобы проверить путь "ключ не найден" наравне с "ключ найден".
    for line in content.lines() {
        if let Some((maybe_key, _)) = line.trim().split_once('=') {
            let _ = pack.get(maybe_key, "en");
            let _ = pack.get(maybe_key, "ru");
            let _ = pack.get_or_key(maybe_key, "xx");
        }
    }
    let _ = pack.get("__definitely_missing_key__", "en");
    let _ = pack.get_or_key("__definitely_missing_key__", "ru");

    let _ = pack.languages();
    let _ = pack.len();
    let _ = pack.is_empty();
    let _ = pack.default_lang();
});
