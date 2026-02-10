mod cleaner;

use cleaner::clean_source_code;
use std::fs;

fn main() {
    let input = fs::read_to_string("src/test.c")
        .expect("Не удалось прочитать файл");

    let result = clean_source_code(&input);

    println!("=== Информационные сообщения ===");
    for msg in &result.messages {
        println!("[INFO] {}", msg);
    }

    if !result.errors.is_empty() {
        println!("\n=== Ошибки ===");
        for err in &result.errors {
            println!("[ERROR] {}", err);
        }
    }

    println!("\n=== Очищенный код ===\n");
    println!("{}", result.cleaned_code);

    fs::write("cleaned_test.c", result.cleaned_code)
        .expect("Не удалось записать файл");

    println!("\nФайл cleaned_test.c сохранён");
}