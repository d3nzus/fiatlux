use anyhow::{Context, bail};
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    Str(String),
    LBrace,
    RBrace,
    Comma,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '#' => {
                while let Some(&c) = chars.peek() {
                    if c == '\n' {
                        break;
                    }
                    chars.next();
                }
            }
            c if c.is_whitespace() => {
                chars.next();
            }
            '{' => {
                tokens.push(Token::LBrace);
                chars.next();
            }
            '}' => {
                tokens.push(Token::RBrace);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        chars.next();
                        break;
                    }
                    s.push(c);
                    chars.next();
                }
                tokens.push(Token::Str(s));
            }
            _ => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() || c == '{' || c == '}' || c == ',' || c == '#' {
                        break;
                    }
                    ident.push(c);
                    chars.next();
                }
                tokens.push(Token::Ident(ident));
            }
        }
    }
    tokens
}

#[derive(Debug)]
pub enum FsNode {
    Dir(String, Vec<FsNode>),
    File(String),
}

#[derive(Debug, Default)]
pub struct BlueprintDef {
    pub directories: Vec<FsNode>,
    pub scripts: Vec<String>,
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

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn expect(&mut self, expected: &Token) -> anyhow::Result<()> {
        match self.next() {
            Some(ref t) if t == expected => Ok(()),
            other => bail!("expected {:?}, found {:?}", expected, other),
        }
    }

    fn parse_blueprint(&mut self) -> anyhow::Result<BlueprintDef> {
        let mut bp = BlueprintDef::default();

        while let Some(tok) = self.peek().cloned() {
            match tok {
                Token::Ident(ref name) if name == "directories" => {
                    self.next();
                    self.expect(&Token::LBrace)?;
                    bp.directories = self.parse_fs_nodes()?;
                }
                Token::Ident(ref name) if name == "scripts" => {
                    self.next();
                    self.expect(&Token::LBrace)?;
                    bp.scripts = self.parse_scripts()?;
                }
                other => bail!("unexpected top-level token: {:?}", other),
            }
        }

        Ok(bp)
    }

    fn parse_fs_nodes(&mut self) -> anyhow::Result<Vec<FsNode>> {
        let mut nodes = Vec::new();

        loop {
            match self.peek().cloned() {
                Some(Token::RBrace) => {
                    self.next();
                    break;
                }
                Some(Token::Ident(name)) => {
                    self.next();
                    if let Some(Token::LBrace) = self.peek() {
                        self.next();
                        let children = self.parse_fs_nodes()?;
                        nodes.push(FsNode::Dir(name, children));
                    } else {
                        nodes.push(FsNode::File(name));
                    }
                }
                other => bail!(
                    "expected identifier or '}}' in directories block, found {:?}",
                    other
                ),
            }
        }

        Ok(nodes)
    }

    fn parse_scripts(&mut self) -> anyhow::Result<Vec<String>> {
        let mut scripts = Vec::new();

        loop {
            match self.next() {
                Some(Token::RBrace) => break,
                Some(Token::Str(s)) => {
                    scripts.push(s);
                    if let Some(Token::Comma) = self.peek() {
                        self.next();
                    }
                }
                other => bail!(
                    "expected string or '}}' in scripts block, found {:?}",
                    other
                ),
            }
        }

        Ok(scripts)
    }
}

pub fn parse_source(source: &str) -> anyhow::Result<BlueprintDef> {
    let tokens = tokenize(source);
    Parser::new(tokens).parse_blueprint()
}

pub fn generate_fs(nodes: &[FsNode], root: &Path) -> anyhow::Result<()> {
    for node in nodes {
        match node {
            FsNode::Dir(name, children) => {
                let dir_path = root.join(name);
                std::fs::create_dir_all(&dir_path)
                    .with_context(|| format!("could not create dir `{}`", dir_path.display()))?;
                generate_fs(children, &dir_path)?;
            }
            FsNode::File(name) => {
                let file_name = if name.contains('.') {
                    name.clone()
                } else {
                    format!("{name}.txt")
                };
                let file_path = root.join(&file_name);
                std::fs::File::create(&file_path)
                    .with_context(|| format!("could not create file `{}`", file_path.display()))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "parser_tests.rs"]
mod tests;
