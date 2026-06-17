use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::env;

// ЛР2: ЛЕКСИЧЕСКИЙ АНАЛИЗАТОР

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword, Identifier, Number, Operator, Delimiter, Type, StringLiteral,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub line: usize,
    pub col: usize,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    // Объединенная регулярка для всех типов токенов.
    let re = Regex::new(r#"(?P<type>int|float|void|char)|(?P<kw>return|if|else|while|for)|(?P<str>"[^"]*")|(?P<id>[a-zA-Z_][a-zA-Z0-9_]*)|(?P<num>\d+\.\d+|\d+)|(?P<op>==|!=|<=|>=|=|\+|-|\*|/|>|<)|(?P<delim>[\(\)\{\};])"#).unwrap();
    
    for (line_idx, line) in input.lines().enumerate() {
        let line_num = line_idx + 1;
        for cap in re.captures_iter(line) {
            let (token_type, value, col) = if let Some(m) = cap.name("type") {
                (TokenType::Type, m.as_str(), m.start() + 1)
            } else if let Some(m) = cap.name("kw") {
                (TokenType::Keyword, m.as_str(), m.start() + 1)
            } else if let Some(m) = cap.name("str") {
                (TokenType::StringLiteral, m.as_str(), m.start() + 1)
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

// ЛР3: СИНТАКСИЧЕСКИЙ АНАЛИЗАТОР (AST)
// Добавить триады просчитанные вручную.

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Int,
    Float,
    Void,
    Char,
    Bool,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Program(Vec<AstNode>),
    Function { name: String, return_type: DataType, body: Box<AstNode>, line: usize },
    VarDecl { name: String, data_type: DataType, value: Option<Box<AstNode>>, line: usize },
    Block(Vec<AstNode>),
    AssignStmt { left: String, right: Box<AstNode>, line: usize },
    IfStmt { condition: Box<AstNode>, then_block: Box<AstNode>, else_block: Option<Box<AstNode>>, line: usize },
    WhileStmt { condition: Box<AstNode>, body: Box<AstNode>, line: usize },
    ReturnStmt { value: Box<AstNode>, line: usize },
    BinaryOp { left: Box<AstNode>, op: String, right: Box<AstNode>, line: usize },
    Literal { value: String, data_type: DataType, line: usize },
    Variable { name: String, line: usize },
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

    fn string_to_datatype(s: &str) -> DataType {
        match s {
            "int" => DataType::Int,
            "float" => DataType::Float,
            "void" => DataType::Void,
            "char" => DataType::Char,
            _ => DataType::Unknown,
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
                let type_token = self.advance().unwrap().clone();
                let name_token = self.expect_type(TokenType::Identifier)?;
                let data_type = Self::string_to_datatype(&type_token.value);
                
                if let Some(next) = self.peek() {
                    if next.value == "(" {
                        return self.parse_function(name_token.value, data_type, type_token.line);
                    }
                }
                
                let value = if let Some(next) = self.peek() {
                    if next.value == "=" {
                        self.advance();
                        Some(Box::new(self.parse_expression()?))
                    } else { None }
                } else { None };
                
                self.expect_value(";")?;
                return Ok(AstNode::VarDecl { name: name_token.value, data_type, value, line: name_token.line });
            }
        }
        self.parse_statement()
    }

    fn parse_function(&mut self, name: String, return_type: DataType, line: usize) -> Result<AstNode, String> {
        self.expect_value("(")?;
        self.expect_value(")")?; 
        let body = self.parse_block()?;
        Ok(AstNode::Function { name, return_type, body: Box::new(body), line })
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
                let data_type = Self::string_to_datatype(&self.advance().unwrap().value);
                let name_token = self.expect_type(TokenType::Identifier)?;
                let value = if let Some(next) = self.peek() {
                    if next.value == "=" {
                        self.advance();
                        Some(Box::new(self.parse_expression()?))
                    } else { None }
                } else { None };
                self.expect_value(";")?;
                Ok(AstNode::VarDecl { name: name_token.value, data_type, value, line: name_token.line })
            }
            TokenType::Keyword => match t.value.as_str() {
                "return" => {
                    let line = t.line;
                    self.advance();
                    let value = self.parse_expression()?;
                    self.expect_value(";")?;
                    Ok(AstNode::ReturnStmt { value: Box::new(value), line })
                }
                "if" => {
                    let line = t.line;
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
                    Ok(AstNode::IfStmt { condition: Box::new(condition), then_block: Box::new(then_block), else_block, line })
                }
                "while" => {
                    let line = t.line;
                    self.advance();
                    self.expect_value("(")?;
                    let condition = self.parse_expression()?;
                    self.expect_value(")")?;
                    let body = self.parse_stmt_or_block()?;
                    Ok(AstNode::WhileStmt { condition: Box::new(condition), body: Box::new(body), line })
                }
                _ => Err(format!("[Ошибка синтаксиса] Стр {}, Кол {}: Неподдерживаемое ключевое слово '{}'", t.line, t.col, t.value)),
            },
            TokenType::Identifier => {
                let name = self.advance().unwrap().value.clone();
                let line = t.line;
                self.expect_value("=")?;
                let expr = self.parse_expression()?;
                self.expect_value(";")?;
                Ok(AstNode::AssignStmt { left: name, right: Box::new(expr), line })
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
        let mut left = self.parse_additive()?;
        while let Some(t) = self.peek().cloned() {
            if ["==", "!=", ">", "<", ">=", "<="].contains(&t.value.as_str()) {
                self.advance();
                let right = self.parse_additive()?;
                left = AstNode::BinaryOp { left: Box::new(left), op: t.value, right: Box::new(right), line: t.line };
            } else { break; }
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_multiplicative()?;
        while let Some(t) = self.peek().cloned() {
            if t.value == "+" || t.value == "-" {
                self.advance();
                let right = self.parse_multiplicative()?;
                left = AstNode::BinaryOp { left: Box::new(left), op: t.value, right: Box::new(right), line: t.line };
            } else { break; }
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_primary()?;
        while let Some(t) = self.peek().cloned() {
            if t.value == "*" || t.value == "/" {
                self.advance();
                let right = self.parse_primary()?;
                left = AstNode::BinaryOp { left: Box::new(left), op: t.value, right: Box::new(right), line: t.line };
            } else { break; }
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<AstNode, String> {
        let t = self.advance().ok_or("Ожидалось выражение")?.clone();
        match t.token_type {
            TokenType::Number => {
                let dtype = if t.value.contains('.') { DataType::Float } else { DataType::Int };
                Ok(AstNode::Literal { value: t.value, data_type: dtype, line: t.line })
            }
            TokenType::StringLiteral => {
                Ok(AstNode::Literal { value: t.value, data_type: DataType::Char, line: t.line })
            }
            TokenType::Identifier => {
                Ok(AstNode::Variable { name: t.value, line: t.line })
            }
            TokenType::Delimiter if t.value == "(" => {
                let node = self.parse_expression()?;
                self.expect_value(")")?;
                Ok(node)
            }
            _ => Err(format!("[Ошибка синтаксиса] Стр {}, Кол {}: Ожидалось число, идентификатор или '(', найдено '{}'", t.line, t.col, t.value)),
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
        AstNode::Function { name, return_type, body, .. } => {
            println!("{}{}Function: {} (returns {:?})", indent, marker, name, return_type);
            let new_indent = indent + if is_last { "    " } else { "│   " };
            print_ast(body, new_indent, true);
        }
        AstNode::VarDecl { name, data_type, value, .. } => {
            print!("{}{}VarDecl: {} [{:?}]", indent, marker, name, data_type);
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
        AstNode::AssignStmt { left, right, .. } => {
            println!("{}{}Assign: {}", indent, marker, left);
            let new_indent = indent + if is_last { "    " } else { "│   " };
            print_ast(right, new_indent, true);
        }
        AstNode::IfStmt { condition, then_block, else_block, .. } => {
            println!("{}{}If", indent, marker);
            let new_indent = indent + if is_last { "    " } else { "│   " };
            print_ast(condition, new_indent.clone(), false);
            if let Some(eb) = else_block {
                print_ast(then_block, new_indent.clone(), false);
                print_ast(eb, new_indent, true);
            } else {
                print_ast(then_block, new_indent, true);
            }
        }
        AstNode::WhileStmt { condition, body, .. } => {
            println!("{}{}While", indent, marker);
            let new_indent = indent + if is_last { "    " } else { "│   " };
            print_ast(condition, new_indent.clone(), false);
            print_ast(body, new_indent, true);
        }
        AstNode::ReturnStmt { value, .. } => {
            println!("{}{}Return", indent, marker);
            let new_indent = indent + if is_last { "    " } else { "│   " };
            print_ast(value, new_indent, true);
        }
        AstNode::BinaryOp { left, op, right, .. } => {
            println!("{}{}BinaryOp: {}", indent, marker, op);
            let new_indent = indent + if is_last { "    " } else { "│   " };
            print_ast(left, new_indent.clone(), false);
            print_ast(right, new_indent, true);
        }
        AstNode::Literal { value, data_type, .. } => {
            println!("{}{}Literal: {} ({:?})", indent, marker, value, data_type);
        }
        AstNode::Variable { name, .. } => {
            println!("{}{}Variable: {}", indent, marker, name);
        }
    }
}

// ЛР4: СЕМАНТИЧЕСКИЙ АНАЛИЗАТОР И ГЕНЕРАЦИЯ ТРИАД

#[derive(Debug, Clone)]
struct SymbolInfo {
    data_type: DataType,
    is_initialized: bool,
    declared_at_line: usize,
    scope: String,
}

struct SemanticAnalyzer {
    symbol_table: HashMap<String, SymbolInfo>,
    errors: Vec<String>,
    triads: Vec<String>,
    triad_count: usize,
    current_scope: String,
}

impl SemanticAnalyzer {
    fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
            errors: Vec::new(),
            triads: Vec::new(),
            triad_count: 0,
            current_scope: "global".to_string(),
        }
    }

    fn next_triad_idx(&self) -> usize {
        self.triad_count + 1
    }

    fn add_triad(&mut self, op: &str, arg1: &str, arg2: &str) -> String {
        self.triad_count += 1;
        self.triads.push(format!("{}: ({}, {}, {})", self.triad_count, op, arg1, arg2));
        format!("^{}", self.triad_count)
    }

    fn patch_triad(&mut self, idx: usize, arg_num: usize, new_val: &str) {
        if let Some(triad) = self.triads.get_mut(idx - 1) {
            // Триада имеет формат "N: (OP, ARG1, ARG2)"
            let parts: Vec<&str> = triad.splitn(2, ": ").collect();
            let content = parts[1].trim_matches(|c| c == '(' || c == ')');
            let mut args: Vec<String> = content.split(", ").map(|s| s.to_string()).collect();
            if arg_num < args.len() {
                args[arg_num] = new_val.to_string();
                *triad = format!("{}: ({}, {}, {})", idx, args[0], args[1], args[2]);
            }
        }
    }

    fn analyze(&mut self, node: &AstNode) -> String {
        match node {
            AstNode::Program(nodes) => {
                for n in nodes { self.analyze(n); }
                "".to_string()
            }
            AstNode::Function { name, body, line: _line, .. } => {
                let old_scope = self.current_scope.clone();
                self.current_scope = name.clone();
                // Функции в таблицу символов (для простоты)
                self.analyze(body);
                self.current_scope = old_scope;
                "".to_string()
            }
            AstNode::VarDecl { name, data_type, value, line } => {
                if self.symbol_table.contains_key(name) {
                    self.errors.push(format!("[Стр {}] Ошибка: Повторное объявление переменной '{}'", line, name));
                } else {
                    let mut is_init = false;
                    let mut init_val = "_".to_string();
                    if let Some(val_node) = value {
                        let val_type = self.get_expr_type(val_node);
                        if val_type != DataType::Unknown && *data_type != val_type {
                            self.errors.push(format!("[Стр {}] Ошибка: Несоответствие типов при инициализации '{}' (ожидалось {:?}, получено {:?})", line, name, data_type, val_type));
                        }
                        is_init = true;
                        init_val = self.analyze(val_node);
                    }
                    self.symbol_table.insert(name.clone(), SymbolInfo {
                        data_type: data_type.clone(),
                        is_initialized: is_init,
                        declared_at_line: *line,
                        scope: self.current_scope.clone(),
                    });
                    if is_init {
                        self.add_triad(":=", name, &init_val);
                    }
                }
                "".to_string()
            }
            AstNode::Block(stmts) => {
                for s in stmts { self.analyze(s); }
                "".to_string()
            }
            AstNode::AssignStmt { left, right, line } => {
                let right_res = self.analyze(right);
                let right_type = self.get_expr_type(right);
                if let Some(info) = self.symbol_table.get_mut(left) {
                    if right_type != DataType::Unknown && info.data_type != right_type {
                        self.errors.push(format!("[Стр {}] Ошибка: Несоответствие типов при присваивании '{}' (ожидалось {:?}, получено {:?})", line, left, info.data_type, right_type));
                    }
                    info.is_initialized = true;
                    self.add_triad(":=", left, &right_res)
                } else {
                    self.errors.push(format!("[Стр {}] Ошибка: Использование необъявленной переменной '{}'", line, left));
                    "".to_string()
                }
            }
            AstNode::IfStmt { condition, then_block, else_block, line: _line } => {
                let cond_res = self.analyze(condition);
                let jf_idx = self.next_triad_idx();
                self.add_triad("JF", &cond_res, "0"); // 0 - placeholder for target
                
                self.analyze(then_block);
                
                if let Some(else_node) = else_block {
                    let jmp_idx = self.next_triad_idx();
                    self.add_triad("JMP", "0", "_"); // 0 - placeholder for target
                    
                    let else_start = self.next_triad_idx();
                    self.patch_triad(jf_idx, 2, &else_start.to_string());
                    
                    self.analyze(else_node);
                    
                    let after_if = self.next_triad_idx();
                    self.patch_triad(jmp_idx, 1, &after_if.to_string());
                } else {
                    let after_if = self.next_triad_idx();
                    self.patch_triad(jf_idx, 2, &after_if.to_string());
                }
                "".to_string()
            }
            AstNode::WhileStmt { condition, body, .. } => {
                let loop_start = self.next_triad_idx();
                let cond_res = self.analyze(condition);
                
                let jf_idx = self.next_triad_idx();
                self.add_triad("JF", &cond_res, "0");
                
                self.analyze(body);
                
                self.add_triad("JMP", &loop_start.to_string(), "_");
                
                let after_loop = self.next_triad_idx();
                self.patch_triad(jf_idx, 2, &after_loop.to_string());
                "".to_string()
            }
            AstNode::ReturnStmt { value, .. } => {
                let res = self.analyze(value);
                self.add_triad("RET", &res, "_")
            }
            AstNode::BinaryOp { left, op, right, line } => {
                let l_res = self.analyze(left);
                let r_res = self.analyze(right);
                
                // Проверка типов
                let lt = self.get_expr_type(left);
                let rt = self.get_expr_type(right);
                if lt != DataType::Unknown && rt != DataType::Unknown && lt != rt {
                     self.errors.push(format!("[Стр {}] Ошибка: Несоответствие типов в операции '{}' ({:?} и {:?})", line, op, lt, rt));
                }
                
                self.add_triad(op, &l_res, &r_res)
            }
            AstNode::Literal { value, .. } => value.clone(),
            AstNode::Variable { name, line } => {
                match self.symbol_table.get(name) {
                    Some(info) => {
                        if !info.is_initialized {
                            self.errors.push(format!("[Стр {}] Ошибка: Использование неинициализированной переменной '{}'", line, name));
                        }
                    }
                    None => {
                        self.errors.push(format!("[Стр {}] Ошибка: Использование необъявленной переменной '{}'", line, name));
                    }
                }
                name.clone()
            }
        }
    }

    fn get_expr_type(&self, node: &AstNode) -> DataType {
        match node {
            AstNode::Literal { data_type, .. } => data_type.clone(),
            AstNode::Variable { name, .. } => {
                self.symbol_table.get(name).map(|info| info.data_type.clone()).unwrap_or(DataType::Unknown)
            }
            AstNode::BinaryOp { left, op, right, .. } => {
                if [">", "<", "==", "!=", ">=", "<="].contains(&op.as_str()) {
                    DataType::Bool
                } else {
                    let lt = self.get_expr_type(left);
                    let rt = self.get_expr_type(right);
                    if lt == rt { lt } else { DataType::Unknown }
                }
            }
            _ => DataType::Unknown,
        }
    }

    fn print_results(&self) {
        println!("\n=== ЛР4: ТАБЛИЦА СИМВОЛОВ ===");
        println!("{:<15} | {:<8} | {:<12} | {:<10} | {:<6}", "Имя", "Тип", "Область", "Инициал.", "Линия");
        println!("{}", "-".repeat(65));
        
        let mut sorted_keys: Vec<_> = self.symbol_table.keys().collect();
        sorted_keys.sort();
        
        for name in sorted_keys {
            let info = &self.symbol_table[name];
            println!("{:<15} | {:<8?} | {:<12} | {:<10} | {:<6}", 
                name, info.data_type, info.scope, if info.is_initialized { "+" } else { "-" }, info.declared_at_line);
        }

        if !self.errors.is_empty() {
            println!("\n=== СЕМАНТИЧЕСКИЕ ОШИБКИ ===");
            for err in &self.errors { println!("[ERROR] {}", err); }
        } else {
            println!("\nСемантический анализ завершен успешно. Ошибок не найдено.");
            println!("\n=== ЛР4: ПРОМЕЖУТОЧНОЕ ПРЕДСТАВЛЕНИЕ (ТРИАДЫ) ===");
            for triad in &self.triads { println!("{}", triad); }
        }
    }
}

// ЛР1: ПРЕПРОЦЕССОР (ОЧИСТКА КОДА)

pub fn clean_source(input: &str) -> String {
    let re_m = Regex::new(r"(?s)/\*.*?\*/").unwrap();
    let re_s = Regex::new(r"//.*").unwrap();
    re_s.replace_all(&re_m.replace_all(input, ""), "").lines()
        .map(|l| l.trim()).filter(|l| !l.is_empty() && !l.starts_with('#')).collect::<Vec<_>>().join("\n")
}

// ТОЧКА ВХОДА

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 { &args[1] } else { "../lab3/src/test.c" };
    
    println!("=== КОМПИЛЯТОР: ЗАПУСК ПОЛНОГО ЦИКЛА (ЛР1-ЛР4) ===");
    println!("Файл: {}", path);

    let input = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => {
            println!("[ERROR] Файл {} не найден.", path);
            return;
        }
    };

    // ЛР1: Препроцессинг
    let cleaned = clean_source(&input);
    println!("\n=== ЛР1: ОЧИЩЕННЫЙ КОД ===");
    println!("{}", cleaned);

    // ЛР2: Лексический анализ
    let tokens = tokenize(&cleaned);
    println!("\n=== ЛР2: РЕЗУЛЬТАТ ТОКЕНИЗАЦИИ ===");
    for t in &tokens {
        println!("{:?} \t| '{}' \t(стр {})", t.token_type, t.value, t.line);
    }

    // ЛР3: Синтаксический анализ
    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(ast) => {
            println!("\n=== ЛР3: АБСТРАКТНОЕ СИНТАКСИЧЕСКОЕ ДЕРЕВО ===");
            print_ast(&ast, "".to_string(), true);

            // ЛР4: Семантический анализ
            let mut analyzer = SemanticAnalyzer::new();
            analyzer.analyze(&ast);
            analyzer.print_results();
        }
        Err(e) => {
            println!("\n=== ОШИБКА СИНТАКСИСА ===");
            println!("{}", e);
        }
    }
}
