use regex::Regex;

pub struct CleanResult {
    pub cleaned_code: String,
    pub messages: Vec<String>,
    pub errors: Vec<String>,
}

/// Проверка на недопустимые символы, можно только ASCII + переводы строк + табы
fn check_invalid_chars(input: &str) -> Vec<String> {
    let mut errors = Vec::new();

    for (i, ch) in input.chars().enumerate() {
        if !(ch.is_ascii() || ch == '\n' || ch == '\t') {
            errors.push(format!(
                "Недопустимый символ '{}' в позиции {}",
                ch, i
            ));
        }
    }

    errors
}

/// проверка на незакрытый комментарии
fn check_unclosed_multiline(input: &str) -> Option<String> {
    let open_count = input.matches("/*").count();
    let close_count = input.matches("*/").count();

    if open_count > close_count {
        Some("Ошибка: незакрытый многострочный комментарий".to_string())
    } else {
        None
    }
}

/// функция очистки
pub fn clean_source_code(input: &str) -> CleanResult {
    let mut messages = Vec::new();
    let mut errors = Vec::new();

    // --- Проверки ---
    if let Some(err) = check_unclosed_multiline(input) {
        errors.push(err);
    }

    errors.extend(check_invalid_chars(input));

    if errors.is_empty() {
        messages.push("Проверка завершена: ошибок не обнаружено".to_string());
    }

    let re_multiline = Regex::new(r"(?s)/\*.*?\*/").unwrap();
    let text = re_multiline.replace_all(input, "");
    messages.push("Удалены многострочные комментарии".to_string());

    let re_single = Regex::new(r"//.*").unwrap();
    let text = re_single.replace_all(&text, "");
    messages.push("Удалены однострочные комментарии".to_string());

    let lines: Vec<String> = text
        .lines()
        .map(|l| l.trim().to_string())
        .collect();

    messages.push("Удалены пробелы и табы по краям строк".to_string());

    let lines: Vec<String> = lines
        .into_iter()
        .filter(|l| !l.is_empty())
        .collect();

    messages.push("Удалены пустые строки".to_string());

    CleanResult {
        cleaned_code: lines.join("\n"),
        messages,
        errors,
    }
}
