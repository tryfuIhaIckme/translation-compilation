use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword,
    Identifier,
    ConstantInt,
    ConstantFloat,
    ConstantString,
    ConstantBool,
    Operator,
    Delimiter,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub lexeme: String,
    pub token_type: TokenType,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    keywords: Vec<&'static str>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            keywords: vec!["int", "void", "char", "if", "else", "while", "for", "return"],
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let res = self.peek();
        if res.is_some() {
            self.pos += 1;
        }
        res
    }

    pub fn tokenize(&mut self) -> (Vec<Token>, Vec<String>) {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
                continue;
            }

            if ch.is_ascii_alphabetic() || ch == '_' {
                tokens.push(self.read_identifier_or_keyword());
            } else if ch.is_ascii_digit() {
                match self.read_number() {
                    Ok(token) => tokens.push(token),
                    Err(err) => errors.push(err),
                }
            } else if ch == '"' {
                match self.read_string() {
                    Ok(token) => tokens.push(token),
                    Err(err) => errors.push(err),
                }
            } else if let Some(token) = self.read_operator_or_delimiter() {
                tokens.push(token);
            } else {
                errors.push(format!("Неизвестный символ '{}' в позиции {}", ch, self.pos));
                self.advance();
            }
        }

        (tokens, errors)
    }

    fn read_identifier_or_keyword(&mut self) -> Token {
        let mut lexeme = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                lexeme.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        let token_type = if self.keywords.contains(&lexeme.as_str()) {
            TokenType::Keyword
        } else if lexeme == "true" || lexeme == "false" {
            TokenType::ConstantBool
        } else {
            TokenType::Identifier
        };

        Token { lexeme, token_type }
    }

    fn read_number(&mut self) -> Result<Token, String> {
        let mut lexeme = String::new();
        let mut dot_count = 0;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                lexeme.push(self.advance().unwrap());
            } else if ch == '.' {
                dot_count += 1;
                lexeme.push(self.advance().unwrap());
                if dot_count > 1 {
                    return Err(format!("Ошибка в числе '{}': лишняя точка", lexeme));
                }
            } else if ch.is_ascii_alphabetic() {
                let bad_char = self.advance().unwrap();
                return Err(format!("Ошибка: буква '{}' в числовой константе", bad_char));
            } else {
                break;
            }
        }

        let token_type = if dot_count == 1 {
            TokenType::ConstantFloat
        } else {
            TokenType::ConstantInt
        };

        Ok(Token { lexeme, token_type })
    }

    fn read_string(&mut self) -> Result<Token, String> {
        self.advance(); // пропускаем "
        let mut lexeme = String::new();
        
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance();
                return Ok(Token { lexeme, token_type: TokenType::ConstantString });
            }
            lexeme.push(self.advance().unwrap());
        }
        
        Err("Ошибка: незакрытый строковый литерал".to_string())
    }

    fn read_operator_or_delimiter(&mut self) -> Option<Token> {
        let operators = ["==", "!=", "<=", ">=", "&&", "||", "=", "+", "-", "*", "/", "<", ">", "!"];
        let delimiters = [';', ',', '(', ')', '{', '}', '[', ']'];

        if let Some(ch) = self.peek() {
            let mut two_chars = String::new();
            two_chars.push(ch);
            if let Some(next_ch) = self.input.get(self.pos + 1) {
                two_chars.push(*next_ch);
                if operators.contains(&two_chars.as_str()) {
                    self.advance();
                    self.advance();
                    return Some(Token { lexeme: two_chars, token_type: TokenType::Operator });
                }
            }

            // Односимвольные
            if operators.contains(&ch.to_string().as_str()) {
                let lexeme = self.advance().unwrap().to_string();
                return Some(Token { lexeme, token_type: TokenType::Operator });
            }

            if delimiters.contains(&ch) {
                let lexeme = self.advance().unwrap().to_string();
                return Some(Token { lexeme, token_type: TokenType::Delimiter });
            }
        }
        None
    }
}