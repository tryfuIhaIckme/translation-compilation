use regex::Regex;

pub struct CleanResult {
    pub cleaned_code: String,
    pub messages: Vec<String>,
    pub errors: Vec<String>,
}

/// Вспомогательная функция для визуализации указателя на ошибку
fn get_error_pointer(input: &str, pos: usize, msg: &str) -> String {
    let line_num = input[..pos].lines().count();
    let line_start = input[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = input[pos..].find('\n').map(|i| i + pos).unwrap_or(input.len());
    let line_content = &input[line_start..line_end];
    let col = pos - line_start;

    format!(
        "{} (строка {}, колонка {})\n{}\n{}^",
        msg, line_num, col + 1, line_content, " ".repeat(col)
    )
}

fn check_invalid_chars(input: &str) -> Vec<String> {
    let mut errors = Vec::new();
    for (i, ch) in input.char_indices() {
        if !(ch.is_ascii() || ch == '\n' || ch == '\t' || ch == '\r') {
            errors.push(get_error_pointer(input, i, &format!("Недопустимый символ '{}'", ch)));
        }
    }
    errors
}

fn check_unclosed_multiline(input: &str) -> Option<String> {
    let open_indices: Vec<_> = input.match_indices("/*").collect();
    let close_count = input.matches("*/").count();

    if open_indices.len() > close_count {
        let last_open_pos = open_indices.last().unwrap().0;
        Some(get_error_pointer(input, last_open_pos, "Ошибка: незакрытый многострочный комментарий"))
    } else {
        None
    }
}

pub fn clean_source_code(input: &str) -> CleanResult {
    let mut messages = Vec::new();
    let mut errors = Vec::new();

    if let Some(err) = check_unclosed_multiline(input) {
        errors.push(err);
    }
    errors.extend(check_invalid_chars(input));

    if errors.is_empty() {
        messages.push("Проверка завершена: ошибок не обнаружено".to_string());
    }

    // Удаление комментариев
    let re_multiline = Regex::new(r"(?s)/\*.*?\*/").unwrap();
    let text = re_multiline.replace_all(input, "");
    let re_single = Regex::new(r"//.*").unwrap();
    let text = re_single.replace_all(&text, "");

    let re_extra_spaces = Regex::new(r"[ \t]+").unwrap();

    let lines: Vec<String> = text
        .lines()
        .map(|l| {
            let trimmed = l.trim();
            re_extra_spaces.replace_all(trimmed, " ").to_string()
        })
        .filter(|l| !l.is_empty())
        .collect();

    messages.push("Удалены комментарии, пустые строки и лишние пробелы".to_string());

    CleanResult {
        cleaned_code: lines.join("\n"),
        messages,
        errors,
    }
}