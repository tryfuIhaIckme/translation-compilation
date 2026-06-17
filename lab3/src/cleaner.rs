use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword, Identifier, Number, Operator, Delimiter, Type,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug)]
pub enum AstNode {
    Program(Vec<AstNode>),
    Function { name: String, return_type: String, body: Box<AstNode> },
    VarDecl { name: String, data_type: String, value: Option<Box<AstNode>> },
    Block(Vec<AstNode>),
    AssignStmt { left: String, right: Box<AstNode> },
    IfStmt { condition: Box<AstNode>, then_block: Box<AstNode>, else_block: Option<Box<AstNode>> },
    WhileStmt { condition: Box<AstNode>, body: Box<AstNode> },
    ReturnStmt { value: Box<AstNode> },
    BinaryExpr { left: Box<AstNode>, op: String, right: Box<AstNode> },
    Literal(String),
    Identifier(String),
}

pub struct CleanResult {
    pub cleaned_code: String,
    pub tokens: Vec<Token>,
    pub ast: Option<AstNode>,
    pub messages: Vec<String>,
    pub errors: Vec<String>,
}

fn get_pointer(input: &str, pos: usize, msg: &str) -> String {
    let line_num = input[..pos].lines().count();
    let line_start = input[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = input[pos..].find('\n').map(|i| i + pos).unwrap_or(input.len());
    let line_content = &input[line_start..line_end];
    let col = pos - line_start;
    format!("{} (стр {}, кол {})\n{}\n{}^", msg, line_num, col + 1, line_content, " ".repeat(col))
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    // Объединенная регулярка для всех типов токенов. Сначала длинные операторы (==, !=, <=, >=)
    let re = Regex::new(r"(?P<type>int|float|void|char)|(?P<kw>return|if|else|while|for)|(?P<id>[a-zA-Z_][a-zA-Z0-9_]*)|(?P<num>\d+\.\d+|\d+)|(?P<op>==|!=|<=|>=|=|\+|-|\*|/|>|<)|(?P<delim>[\(\)\{\};])").unwrap();
    
    for (line_idx, line) in input.lines().enumerate() {
        let line_num = line_idx + 1;
        for cap in re.captures_iter(line) {
            let (token_type, value, col) = if let Some(m) = cap.name("type") {
                (TokenType::Type, m.as_str(), m.start() + 1)
            } else if let Some(m) = cap.name("kw") {
                (TokenType::Keyword, m.as_str(), m.start() + 1)
            } else if let Some(m) = cap.name("id") {
                (TokenType::Identifier, m.as_str(), m.start() + 1)
            } else if let Some(m) = cap.name("num") {
                (TokenType::Number, m.as_str(), m.start() + 1)
            } else if let Some(m) = cap.name("op") {
                (TokenType::Operator, m.as_str(), m.start() + 1)
            } else if let Some(m) = cap.name("delim") {
                (TokenType::Delimiter, m.as_str(), m.start() + 1)
            } else { continue; };

            tokens.push(Token {
                token_type,
                value: value.to_string(),
                line: line_num,
                col,
            });
        }
    }
    tokens
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        if t.is_some() { self.pos += 1; }
        t
    }

    fn expect_type(&mut self, t_type: TokenType) -> Result<Token, String> {
        let t = self.peek().ok_or("Неожиданный конец файла")?;
        if t.token_type == t_type {
            Ok(self.advance().unwrap().clone())
        } else {
            Err(format!("[Ошибка синтаксиса] Стр {}, Кол {}: Ожидался тип {:?}, найдено '{}'", t.line, t.col, t_type, t.value))
        }
    }

    fn expect_value(&mut self, val: &str) -> Result<Token, String> {
        let t = self.peek().ok_or("Неожиданный конец файла")?;
        if t.value == val {
            Ok(self.advance().unwrap().clone())
        } else {
            Err(format!("[Ошибка синтаксиса] Стр {}, Кол {}: Ожидалось '{}', найдено '{}'", t.line, t.col, val, t.value))
        }
    }

    fn parse_program(&mut self) -> Result<AstNode, String> {
        let mut nodes = Vec::new();
        while self.pos < self.tokens.len() {
            nodes.push(self.parse_top_level()?);
        }
        Ok(AstNode::Program(nodes))
    }

    fn parse_top_level(&mut self) -> Result<AstNode, String> {
        if let Some(t) = self.peek() {
            if t.token_type == TokenType::Type {
                let data_type = self.advance().unwrap().clone();
                let name = self.expect_type(TokenType::Identifier)?;
                
                if let Some(next) = self.peek() {
                    if next.value == "(" {
                        return self.parse_function(name.value, data_type.value);
                    }
                }
                
                let value = if let Some(next) = self.peek() {
                    if next.value == "=" {
                        self.advance();
                        Some(Box::new(self.parse_expression()?))
                    } else { None }
                } else { None };
                
                self.expect_value(";")?;
                return Ok(AstNode::VarDecl { name: name.value, data_type: data_type.value, value });
            }
        }
        self.parse_statement()
    }

    fn parse_function(&mut self, name: String, return_type: String) -> Result<AstNode, String> {
        self.expect_value("(")?;
        self.expect_value(")")?; // Для простоты без параметров
        let body = self.parse_block()?;
        Ok(AstNode::Function { name, return_type, body: Box::new(body) })
    }

    fn parse_block(&mut self) -> Result<AstNode, String> {
        self.expect_value("{")?;
        let mut statements = Vec::new();
        while let Some(t) = self.peek() {
            if t.value == "}" { break; }
            statements.push(self.parse_statement()?);
        }
        self.expect_value("}")?;
        Ok(AstNode::Block(statements))
    }

    fn parse_statement(&mut self) -> Result<AstNode, String> {
        let t = self.peek().ok_or("Неожиданный конец файла")?.clone();
        match t.token_type {
            TokenType::Type => {
                let data_type = self.advance().unwrap().value.clone();
                let name = self.expect_type(TokenType::Identifier)?.value;
                let value = if let Some(next) = self.peek() {
                    if next.value == "=" {
                        self.advance();
                        Some(Box::new(self.parse_expression()?))
                    } else { None }
                } else { None };
                self.expect_value(";")?;
                Ok(AstNode::VarDecl { name, data_type, value })
            }
            TokenType::Keyword => match t.value.as_str() {
                "return" => {
                    self.advance();
                    let value = self.parse_expression()?;
                    self.expect_value(";")?;
                    Ok(AstNode::ReturnStmt { value: Box::new(value) })
                }
                "if" => {
                    self.advance();
                    self.expect_value("(")?;
                    let condition = self.parse_expression()?;
                    self.expect_value(")")?;
                    let then_block = self.parse_stmt_or_block()?;
                    let mut else_block = None;
                    if let Some(next) = self.peek() {
                        if next.value == "else" {
                            self.advance();
                            else_block = Some(Box::new(self.parse_stmt_or_block()?));
                        }
                    }
                    Ok(AstNode::IfStmt { condition: Box::new(condition), then_block: Box::new(then_block), else_block })
                }
                "while" => {
                    self.advance();
                    self.expect_value("(")?;
                    let condition = self.parse_expression()?;
                    self.expect_value(")")?;
                    let body = self.parse_stmt_or_block()?;
                    Ok(AstNode::WhileStmt { condition: Box::new(condition), body: Box::new(body) })
                }
                _ => Err(format!("[Ошибка синтаксиса] Стр {}, Кол {}: Неподдерживаемое ключевое слово '{}'", t.line, t.col, t.value)),
            },
            TokenType::Identifier => {
                let name = self.advance().unwrap().value.clone();
                self.expect_value("=")?;
                let expr = self.parse_expression()?;
                self.expect_value(";")?;
                Ok(AstNode::AssignStmt { left: name, right: Box::new(expr) })
            }
            _ => Err(format!("[Ошибка синтаксиса] Стр {}, Кол {}: Неожиданный токен '{}'", t.line, t.col, t.value)),
        }
    }

    fn parse_stmt_or_block(&mut self) -> Result<AstNode, String> {
        if let Some(t) = self.peek() {
            if t.value == "{" {
                return self.parse_block();
            }
        }
        self.parse_statement()
    }

    fn parse_expression(&mut self) -> Result<AstNode, String> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<AstNode, String> {
        let mut node = self.parse_additive()?;
        while let Some(t) = self.peek() {
            if ["==", "!=", ">", "<", ">=", "<="].contains(&t.value.as_str()) {
                let op = self.advance().unwrap().value.clone();
                let right = self.parse_additive()?;
                node = AstNode::BinaryExpr { left: Box::new(node), op, right: Box::new(right) };
            } else { break; }
        }
        Ok(node)
    }

    fn parse_additive(&mut self) -> Result<AstNode, String> {
        let mut node = self.parse_multiplicative()?;
        while let Some(t) = self.peek() {
            if t.value == "+" || t.value == "-" {
                let op = self.advance().unwrap().value.clone();
                let right = self.parse_multiplicative()?;
                node = AstNode::BinaryExpr { left: Box::new(node), op, right: Box::new(right) };
            } else { break; }
        }
        Ok(node)
    }

    fn parse_multiplicative(&mut self) -> Result<AstNode, String> {
        let mut node = self.parse_primary()?;
        while let Some(t) = self.peek() {
            if t.value == "*" || t.value == "/" {
                let op = self.advance().unwrap().value.clone();
                let right = self.parse_primary()?;
                node = AstNode::BinaryExpr { left: Box::new(node), op, right: Box::new(right) };
            } else { break; }
        }
        Ok(node)
    }

    fn parse_primary(&mut self) -> Result<AstNode, String> {
        let t = self.advance().ok_or("Ожидалось выражение")?.clone();
        match t.token_type {
            TokenType::Number => Ok(AstNode::Literal(t.value)),
            TokenType::Identifier => Ok(AstNode::Identifier(t.value)),
            TokenType::Delimiter if t.value == "(" => {
                let node = self.parse_expression()?;
                self.expect_value(")")?;
                Ok(node)
            }
            _ => Err(format!("[Ошибка синтаксиса] Стр {}, Кол {}: Ожидалось число, идентификатор или '(' , найдено '{}'", t.line, t.col, t.value)),
        }
    }
}

pub fn print_ast(node: &AstNode, indent: String, is_last: bool) {
    let marker = if is_last { "└── " } else { "├── " };
    match node {
        AstNode::Program(nodes) => {
            println!("Program");
            for (i, n) in nodes.iter().enumerate() {
                print_ast(n, "".to_string(), i == nodes.len() - 1);
            }
        }
        AstNode::Function { name, return_type, body } => {
            println!("{}{}Function: {} (returns {})", indent, marker, name, return_type);
            let new_indent = indent + if is_last { "    " } else { "│   " };
            print_ast(body, new_indent, true);
        }
        AstNode::VarDecl { name, data_type, value } => {
            print!("{}{}VarDecl: {} [{}]", indent, marker, name, data_type);
            if let Some(v) = value {
                println!(" =");
                let new_indent = indent + if is_last { "    " } else { "│   " };
                print_ast(v, new_indent, true);
            } else {
                println!();
            }
        }
        AstNode::Block(stmts) => {
            println!("{}{}Block", indent, marker);
            let new_indent = indent + if is_last { "    " } else { "│   " };
            for (i, s) in stmts.iter().enumerate() {
                print_ast(s, new_indent.clone(), i == stmts.len() - 1);
            }
        }
        AstNode::AssignStmt { left, right } => {
            println!("{}{}Assign: {}", indent, marker, left);
            let new_indent = indent + if is_last { "    " } else { "│   " };
            print_ast(right, new_indent, true);
        }
        AstNode::IfStmt { condition, then_block, else_block } => {
            println!("{}{}If", indent, marker);
            let new_indent = indent + if is_last { "    " } else { "│   " };
            print_ast(condition, new_indent.clone(), false);
            if else_block.is_some() {
                print_ast(then_block, new_indent.clone(), false);
                print_ast(else_block.as_ref().unwrap(), new_indent, true);
            } else {
                print_ast(then_block, new_indent, true);
            }
        }
        AstNode::WhileStmt { condition, body } => {
            println!("{}{}While", indent, marker);
            let new_indent = indent + if is_last { "    " } else { "│   " };
            print_ast(condition, new_indent.clone(), false);
            print_ast(body, new_indent, true);
        }
        AstNode::ReturnStmt { value } => {
            println!("{}{}Return", indent, marker);
            let new_indent = indent + if is_last { "    " } else { "│   " };
            print_ast(value, new_indent, true);
        }
        AstNode::BinaryExpr { left, op, right } => {
            println!("{}{}BinaryExpr: {}", indent, marker, op);
            let new_indent = indent + if is_last { "    " } else { "│   " };
            print_ast(left, new_indent.clone(), false);
            print_ast(right, new_indent, true);
        }
        AstNode::Literal(val) => {
            println!("{}{}Literal: {}", indent, marker, val);
        }
        AstNode::Identifier(name) => {
            println!("{}{}Identifier: {}", indent, marker, name);
        }
    }
}

pub fn clean_source_code(input: &str) -> CleanResult {
    let mut errors = Vec::new();
    let mut messages = Vec::new();

    if input.matches("/*").count() > input.matches("*/").count() {
        let pos = input.find("/*").unwrap();
        errors.push(get_pointer(input, pos, "Незакрытый комментарий"));
    }
    for (i, ch) in input.char_indices() {
        if !(ch.is_ascii() || ch == '\n' || ch == '\t' || ch == '\r') {
            errors.push(get_pointer(input, i, "Запрещенный символ"));
        }
    }

    // Очистка от комментариев
    let re_m = Regex::new(r"(?s)/\*.*?\*/").unwrap();
    let re_s = Regex::new(r"//.*").unwrap();
    let cleaned = re_s.replace_all(&re_m.replace_all(input, ""), "").lines()
        .map(|l| l.trim()).filter(|l| !l.is_empty() && !l.starts_with('#')).collect::<Vec<_>>().join("\n");

    // Токенизация
    let tokens = tokenize(&cleaned);
    
    // Парсинг
    let mut parser = Parser::new(tokens.clone());
    let ast = match parser.parse_program() {
        Ok(node) => Some(node),
        Err(e) => {
            errors.push(e);
            None
        }
    };

    if errors.is_empty() { 
        messages.push("Очистка от комментариев завершена".to_string());
        messages.push("Лексический анализ (ЛР2) завершен".to_string());
        messages.push("Синтаксическое дерево (ЛР3) построено".to_string());
    }

    CleanResult { cleaned_code: cleaned, tokens, ast, messages, errors }
}