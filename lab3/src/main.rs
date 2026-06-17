mod cleaner;
use cleaner::{clean_source_code, print_ast};
use std::fs;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 { &args[1] } else { "src/test.c" };
    
    println!("=== АНАЛИЗ ФАЙЛА: {} ===", path);
    let input = fs::read_to_string(path).unwrap_or_else(|_| panic!("Файл {} не найден", path));

    let result = clean_source_code(&input);

    println!("=== ИНФОРМАЦИОННЫЕ СООБЩЕНИЯ ===");
    for msg in &result.messages {
        println!("[INFO] {}", msg);
    }

    if !result.errors.is_empty() {
        println!("\n=== ОШИБКИ ===");
        for err in &result.errors { println!("[ERROR] {}", err); }
        return;
    }

    println!("\n=== ЛР2: РЕЗУЛЬТАТ ТОКЕНИЗАЦИИ ===");
    for t in &result.tokens {
        println!("{:?} \t| '{}' \t(стр {})", t.token_type, t.value, t.line);
    }

    println!("\n=== ЛР3: АБСТРАКТНОЕ СИНТАКСИЧЕСКОЕ ДЕРЕВО ===");
    if let Some(ast) = &result.ast {
        print_ast(ast, "".to_string(), true);
    }

    println!("\nСинтаксический анализ завершён успешно. Ошибок не найдено.");

    fs::write("cleaned_test.c", &result.cleaned_code).unwrap();
}