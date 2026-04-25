mod cleaner;

use cleaner::clean_source_code;
use std::fs;
use std::process::Command;

fn check_syntax_after_cleaning(code: &str) -> Vec<String> {
    let mut syntax_errors = Vec::new();
    let lines: Vec<&str> = code.lines().collect();

    let mut brace_balance = 0;
    
    for (i, line) in lines.iter().enumerate() {
        let l = line.trim();
        
        if !l.is_empty() && 
           !l.ends_with('{') && !l.ends_with('}') && !l.ends_with(';') && 
           !l.starts_with('#') && !l.starts_with("if") && !l.contains("main") {
            
            let pointer = format!("{}^", " ".repeat(l.len().saturating_sub(1)));
            syntax_errors.push(format!(
                "Возможно, пропущена ';' в строке {}:\n{}\n{}", 
                i + 1, l, pointer
            ));
        }

        brace_balance += l.matches('{').count() as i32;
        brace_balance -= l.matches('}').count() as i32;
    }

    if brace_balance != 0 {
        syntax_errors.push(format!("Ошибка: нарушен баланс фигурных скобок ({})", brace_balance));
    }

    syntax_errors
}

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

    let syntax_errors = check_syntax_after_cleaning(&result.cleaned_code);
    if !syntax_errors.is_empty() {
        println!("\n=== Анализ синтаксиса перед компиляцией ===");
        for err in syntax_errors {
            println!("[SYNTAX ERROR] {}", err);
        }
        println!("\n[STOP] Исправьте ошибки перед запуском GCC.");
        return;
    }

    let cleaned_file = "cleaned_test.c";
    fs::write(cleaned_file, &result.cleaned_code)
        .expect("Не удалось записать файл");

    println!("\nФайл {} сохранён", cleaned_file);

    let output_binary = if cfg!(windows) { "cleaned_test.exe" } else { "cleaned_test" };

    println!("[INFO] Запуск компилятора GCC...");
    let compile_status = Command::new("gcc")
        .arg(cleaned_file)
        .arg("-o")
        .arg(output_binary)
        .status()
        .expect("Не удалось вызвать gcc");

    if !compile_status.success() {
        println!("[ERROR] Компиляция не удастся");
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