mod cleaner;

use cleaner::clean_source_code;
use std::fs;
use std::process::Command;

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

    let cleaned_file = "cleaned_test.c";
    fs::write(cleaned_file, &result.cleaned_code)
        .expect("Не удалось записать файл");

    println!("\nФайл {} сохранён", cleaned_file);

    let output_binary = if cfg!(windows) {
        "cleaned_test.exe"
    } else {
        "cleaned_test"
    };

    let compile_status = Command::new("gcc")
        .arg(cleaned_file)
        .arg("-o")
        .arg(output_binary)
        .status()
        .expect("Не удалось вызвать gcc");

    if !compile_status.success() {
        println!("[ERROR] Компиляция не удалась");
        return;
    }

    println!("[INFO] Компиляция прошла успешно");

    let run_status = Command::new(format!("./{}", output_binary))
        .status()
        .expect("Не удалось запустить скомпилированный файл");

    if !run_status.success() {
        println!("[ERROR] Программа завершилась с ошибкой");
    }
}