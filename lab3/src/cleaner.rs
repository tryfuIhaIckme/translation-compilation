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
}

#[derive(Debug)]
pub enum AstNode {
    Program(Vec<AstNode>),
    VarDecl { name: String, data_type: String },
    AssignStmt { left: String, right: String },
    ReturnStmt { value: String },
    Function { name: String, return_type: String, body: Vec<AstNode> },
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

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let re_keywords = Regex::new(r"^(return|if|else|while|for)$").unwrap();
    let re_types = Regex::new(r"^(int|float|void|char)$").unwrap();
    let re_ident = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    let re_num = Regex::new(r"^[0-9]+$").unwrap();

    for (i, line) in input.lines().enumerate() {
        let line_num = i + 1;
        if line.trim().starts_with('#') { continue; }
        
        let parts = line.replace("(", " ( ").replace(")", " ) ").replace("{", " { ").replace("}", " } ").replace(";", " ; ").replace("=", " = ");
        for word in parts.split_whitespace() {
            let token_type = if re_types.is_match(word) { TokenType::Type }
            else if re_keywords.is_match(word) { TokenType::Keyword }
            else if re_ident.is_match(word) { TokenType::Identifier }
            else if re_num.is_match(word) { TokenType::Number }
            else if "=+-*/".contains(word) { TokenType::Operator }
            else { TokenType::Delimiter };

            tokens.push(Token { token_type, value: word.to_string(), line: line_num });
        }
    }
    tokens
}

struct Parser { tokens: Vec<Token>, pos: usize }
impl Parser {
    fn new(tokens: Vec<Token>) -> Self { Self { tokens, pos: 0 } }
    fn peek(&self) -> Option<&Token> { self.tokens.get(self.pos) }
    fn eat(&mut self, exp: &str) -> Result<Token, String> {
        let t = self.tokens.get(self.pos).ok_or("Конец файла")?;
        if !exp.is_empty() && t.value != exp { return Err(format!("Ошибка в стр {}: ждали '{}', нашли '{}'", t.line, exp, t.value)); }
        self.pos += 1; Ok(t.clone())
    }
    fn parse_program(&mut self) -> Result<AstNode, String> {
        let mut nodes = Vec::new();
        while self.pos < self.tokens.len() { nodes.push(self.parse_top()?); }
        Ok(AstNode::Program(nodes))
    }
    fn parse_top(&mut self) -> Result<AstNode, String> {
        let t_type = self.eat("")?; let t_name = self.eat("")?;
        if self.peek().map(|t| t.value == "(").unwrap_or(false) {
            self.eat("(")?; self.eat(")")?; self.eat("{")?;
            let mut body = Vec::new();
            while self.peek().map(|t| t.value != "}").unwrap_or(false) { body.push(self.parse_stmt()?); }
            self.eat("}")?;
            Ok(AstNode::Function { name: t_name.value, return_type: t_type.value, body })
        } else {
            self.eat(";")?; Ok(AstNode::VarDecl { name: t_name.value, data_type: t_type.value })
        }
    }
    fn parse_stmt(&mut self) -> Result<AstNode, String> {
        let t = self.eat("")?;
        if t.value == "return" {
            let v = self.eat("")?; self.eat(";")?; Ok(AstNode::ReturnStmt { value: v.value })
        } else if t.token_type == TokenType::Type {
            let n = self.eat("")?; self.eat(";")?; Ok(AstNode::VarDecl { name: n.value, data_type: t.value })
        } else if t.token_type == TokenType::Identifier {
            self.eat("=")?; let v = self.eat("")?; self.eat(";")?; Ok(AstNode::AssignStmt { left: t.value, right: v.value })
        } else { Err(format!("Ошибка в стр {}: '{}'", t.line, t.value)) }
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
            for (i, n) in body.iter().enumerate() {
                print_ast(n, new_indent.clone(), i == body.len() - 1);
            }
        }
        AstNode::VarDecl { name, data_type } => {
            println!("{}{}VarDecl: {} [{}]", indent, marker, name, data_type);
        }
        AstNode::AssignStmt { left, right } => {
            println!("{}{}Assign: {} = {}", indent, marker, left, right);
        }
        AstNode::ReturnStmt { value } => {
            println!("{}{}Return: {}", indent, marker, value);
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

    let re_m = Regex::new(r"(?s)/\*.*?\*/").unwrap();
    let re_s = Regex::new(r"//.*").unwrap();
    let cleaned = re_s.replace_all(&re_m.replace_all(input, ""), "").lines()
        .map(|l| l.trim()).filter(|l| !l.is_empty() && !l.starts_with('#')).collect::<Vec<_>>().join("\n");

    let tokens = tokenize(&cleaned);
    let mut parser = Parser::new(tokens.clone());
    let ast = parser.parse_program().map_err(|e| errors.push(e)).ok();

    if errors.is_empty() { 
        messages.push("Очистка от комментариев завершена".to_string());
        messages.push("Лексический анализ (ЛР2) завершен".to_string());
        messages.push("Синтаксическое дерево (ЛР3) построено".to_string());
    }

    CleanResult { cleaned_code: cleaned, tokens, ast, messages, errors }
}