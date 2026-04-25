mod lexer;

use std::fs;
use regex::Regex;
use lexer::{Lexer, TokenType};

fn clean_code(input: &str) -> String {
    let re_multiline = Regex::new(r"(?s)/\*.*?\*/").unwrap();
    let text = re_multiline.replace_all(input, "");
    let re_single = Regex::new(r"//.*").unwrap();
    let text = re_single.replace_all(&text, "");
    
    text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn main() {
    let file_path = "../lab1/src/test.c";
    let input = fs::read_to_string(file_path)
        .expect("Не удалось прочитать исходный файл test.c");

    println!("Очистка кода ");
    let cleaned = clean_code(&input);
    println!("{}", cleaned);

    println!("\n Лексический анализ ");
    let mut lexer = Lexer::new(&cleaned);
    let (tokens, errors) = lexer.tokenize();

    println!("{:<15} | {:<20}", "Лексема", "Тип");
    println!("{:-<16}+{:-<21}", "", "");
    for token in &tokens {
        println!("{:<15} | {:<20}", token.lexeme, token.token_type.to_string());
    }

    // Вывод списка объектов
    println!("\nПоследовательность токенов:");
    let token_list: Vec<(String, String)> = tokens.iter()
        .map(|t| (t.token_type.to_string(), t.lexeme.clone()))
        .collect();
    println!("{:?}", token_list);

    if !errors.is_empty() {
        println!("\n=== ОБНАРУЖЕНЫ ОШИБКИ ===");
        for err in errors {
            println!("[ERROR] {}", err);
        }
    } else {
        println!("\nЛексический анализ завершён успешно. Обнаружено {} токенов. Ошибок не найдено.", tokens.len());
    }
}