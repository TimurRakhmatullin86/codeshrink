use std::path::{Path, PathBuf};
use ignore::WalkBuilder;
use rayon::prelude::*;
use tree_sitter::{Parser, Tree, Node};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    TypeScript,
    Tsx,
    JavaScript,
    Python,
    Rust,
    Go,
    Java,
}

impl Language {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "ts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            "js" | "mjs" | "cjs" => Some(Self::JavaScript),
            "py" => Some(Self::Python),
            "rs" => Some(Self::Rust),
            "go" => Some(Self::Go),
            "java" => Some(Self::Java),
            _ => None,
        }
    }

    fn tree_sitter_language(&self) -> tree_sitter::Language {
        match self {
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::Go => tree_sitter_go::LANGUAGE.into(),
            Self::Java => tree_sitter_java::LANGUAGE.into(),
        }
    }

    fn definition_node_kinds(&self) -> &[&str] {
        match self {
            Self::TypeScript | Self::Tsx | Self::JavaScript => &[
                "function_declaration",
                "method_definition",
                "class_declaration",
                "interface_declaration",
                "type_alias_declaration",
                "enum_declaration",
                "arrow_function",
                "variable_declarator",
                "export_statement",
            ],
            Self::Python => &[
                "function_definition",
                "class_definition",
                "decorated_definition",
            ],
            Self::Rust => &[
                "function_item",
                "struct_item",
                "enum_item",
                "trait_item",
                "impl_item",
                "type_item",
                "const_item",
                "static_item",
                "mod_item",
            ],
            Self::Go => &[
                "function_declaration",
                "method_declaration",
                "type_declaration",
                "type_spec",
            ],
            Self::Java => &[
                "method_declaration",
                "class_declaration",
                "interface_declaration",
                "enum_declaration",
                "constructor_declaration",
            ],
        }
    }

    fn import_node_kinds(&self) -> &[&str] {
        match self {
            Self::TypeScript | Self::Tsx | Self::JavaScript => &[
                "import_statement",
                "import_clause",
            ],
            Self::Python => &[
                "import_statement",
                "import_from_statement",
            ],
            Self::Rust => &[
                "use_declaration",
                "extern_crate_declaration",
            ],
            Self::Go => &[
                "import_declaration",
            ],
            Self::Java => &[
                "import_declaration",
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Class,
    Interface,
    Enum,
    Type,
    Method,
    Struct,
    Trait,
    Impl,
    Module,
    Variable,
    Import,
    Other,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub start_line: usize,
    pub end_line: usize,
    pub signature: String,
}

#[derive(Debug)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub language: Language,
    pub source: String,
    pub line_count: usize,
    pub symbols: Vec<Symbol>,
    pub imports: Vec<Symbol>,
}

pub fn scan_repo(repo_path: &Path) -> Result<Vec<ParsedFile>, std::io::Error> {
    let walker = WalkBuilder::new(repo_path)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !matches!(
                name.as_ref(),
                "node_modules" | "target" | ".git" | "dist" | "build"
                | "__pycache__" | ".venv" | "venv" | "vendor"
            )
        })
        .build();

    let entries: Vec<(PathBuf, Language)> = walker
        .flatten()
        .filter(|e| e.file_type().is_some_and(|ft| ft.is_file()))
        .filter_map(|e| {
            let path = e.path().to_path_buf();
            let ext = path.extension()?.to_str()?;
            let lang = Language::from_extension(ext)?;
            Some((path, lang))
        })
        .collect();

    let files: Vec<ParsedFile> = entries
        .par_iter()
        .filter_map(|(path, lang)| {
            let source = std::fs::read_to_string(path).ok()?;
            let line_count = source.lines().count();
            let tree = parse_source(&source, *lang)?;
            let (symbols, imports) = extract_symbols(&source, tree.root_node(), *lang);
            Some(ParsedFile {
                path: path.clone(),
                language: *lang,
                source,
                line_count,
                symbols,
                imports,
            })
        })
        .collect();

    Ok(files)
}

fn parse_source(source: &str, lang: Language) -> Option<Tree> {
    let mut parser = Parser::new();
    parser.set_language(&lang.tree_sitter_language()).ok()?;
    parser.parse(source, None)
}

fn extract_symbols(source: &str, root: Node, lang: Language) -> (Vec<Symbol>, Vec<Symbol>) {
    let mut symbols = Vec::new();
    let mut imports = Vec::new();
    let def_kinds = lang.definition_node_kinds();
    let import_kinds = lang.import_node_kinds();

    walk_tree(root, source, def_kinds, import_kinds, &mut symbols, &mut imports);
    (symbols, imports)
}

fn walk_tree(
    node: Node,
    source: &str,
    def_kinds: &[&str],
    import_kinds: &[&str],
    symbols: &mut Vec<Symbol>,
    imports: &mut Vec<Symbol>,
) {
    let kind = node.kind();

    if import_kinds.contains(&kind) {
        let text = node_text(node, source);
        imports.push(Symbol {
            name: text.clone(),
            kind: SymbolKind::Import,
            start_line: node.start_position().row,
            end_line: node.end_position().row,
            signature: text,
        });
    }

    if def_kinds.contains(&kind) {
        let name = find_name_child(node, source).unwrap_or_default();
        if !name.is_empty() {
            let sig = extract_signature(node, source);
            let sym_kind = classify_kind(kind);
            symbols.push(Symbol {
                name,
                kind: sym_kind,
                start_line: node.start_position().row,
                end_line: node.end_position().row,
                signature: sig,
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_tree(child, source, def_kinds, import_kinds, symbols, imports);
    }
}

fn find_name_child(node: Node, source: &str) -> Option<String> {
    // Look for a child with field name "name" first
    if let Some(name_node) = node.child_by_field_name("name") {
        return Some(node_text(name_node, source));
    }
    // Fallback: first identifier child
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier"
            || child.kind() == "type_identifier"
            || child.kind() == "property_identifier"
        {
            return Some(node_text(child, source));
        }
    }
    None
}

fn extract_signature(node: Node, source: &str) -> String {
    let start = node.start_byte();
    let text = &source[start..];
    let end = text
        .find('{')
        .or_else(|| text.find('\n'))
        .unwrap_or(text.len())
        .min(200);
    // Ensure we don't split a multi-byte char
    let safe_end = if text.is_char_boundary(end) {
        end
    } else {
        text.floor_char_boundary(end)
    };
    text[..safe_end].trim().to_string()
}

fn classify_kind(node_kind: &str) -> SymbolKind {
    match node_kind {
        k if k.contains("function") || k.contains("arrow_function") => SymbolKind::Function,
        k if k.contains("method") => SymbolKind::Method,
        k if k.contains("class") => SymbolKind::Class,
        k if k.contains("interface") => SymbolKind::Interface,
        k if k.contains("enum") => SymbolKind::Enum,
        k if k.contains("struct") => SymbolKind::Struct,
        k if k.contains("trait") => SymbolKind::Trait,
        k if k.contains("impl") => SymbolKind::Impl,
        k if k.contains("type") => SymbolKind::Type,
        k if k.contains("mod") => SymbolKind::Module,
        k if k.contains("variable") || k.contains("const") || k.contains("static") => SymbolKind::Variable,
        _ => SymbolKind::Other,
    }
}

fn node_text(node: Node, source: &str) -> String {
    source[node.byte_range()].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_file(dir: &Path, name: &str, content: &str) {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn parse_typescript_symbols() {
        let dir = TempDir::new().unwrap();
        write_file(dir.path(), "auth.ts", r#"
import { Request } from 'express';

export function authenticate(req: Request): boolean {
    const token = req.headers.authorization;
    if (!token) return false;
    return verifyToken(token);
}

function verifyToken(token: string): boolean {
    return token.startsWith('Bearer ');
}

export class AuthService {
    private secret: string;
    constructor(secret: string) {
        this.secret = secret;
    }
    validate(token: string): boolean {
        return true;
    }
}

export interface AuthConfig {
    secret: string;
    expiry: number;
}
"#);

        let files = scan_repo(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
        let f = &files[0];
        assert_eq!(f.language, Language::TypeScript);
        assert!(f.symbols.len() >= 3, "expected >=3 symbols, got {}: {:?}",
            f.symbols.len(), f.symbols.iter().map(|s| &s.name).collect::<Vec<_>>());

        let names: Vec<&str> = f.symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"authenticate"), "missing authenticate in {:?}", names);
        assert!(names.contains(&"AuthService"), "missing AuthService in {:?}", names);
        assert!(names.contains(&"AuthConfig"), "missing AuthConfig in {:?}", names);
    }

    #[test]
    fn parse_python_symbols() {
        let dir = TempDir::new().unwrap();
        write_file(dir.path(), "handler.py", r#"
import os
from typing import Optional

def process_request(data: dict) -> dict:
    return {"status": "ok"}

class RequestHandler:
    def __init__(self, config):
        self.config = config

    def handle(self, req):
        return self.process(req)
"#);

        let files = scan_repo(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
        let names: Vec<&str> = files[0].symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"process_request"));
        assert!(names.contains(&"RequestHandler"));
    }

    #[test]
    fn parse_rust_symbols() {
        let dir = TempDir::new().unwrap();
        write_file(dir.path(), "lib.rs", r#"
use std::collections::HashMap;

pub struct Config {
    pub name: String,
}

pub fn initialize(config: &Config) -> Result<(), Error> {
    Ok(())
}

pub trait Handler {
    fn handle(&self, req: Request) -> Response;
}

impl Handler for Config {
    fn handle(&self, req: Request) -> Response {
        todo!()
    }
}
"#);

        let files = scan_repo(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
        let names: Vec<&str> = files[0].symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Config"), "missing Config in {:?}", names);
        assert!(names.contains(&"initialize"), "missing initialize in {:?}", names);
        assert!(names.contains(&"Handler"), "missing Handler in {:?}", names);
    }

    #[test]
    fn skips_gitignored_dirs() {
        let dir = TempDir::new().unwrap();
        // ignore crate needs .git to honour .gitignore
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        write_file(dir.path(), ".gitignore", "ignored/\n");
        write_file(dir.path(), "good.ts", "export function good() {}");
        write_file(dir.path(), "ignored/bad.ts", "export function bad() {}");
        write_file(dir.path(), "node_modules/pkg.ts", "export function pkg() {}");

        let files = scan_repo(dir.path()).unwrap();
        let paths: Vec<String> = files.iter()
            .map(|f| f.path.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(paths.contains(&"good.ts".to_string()));
        assert!(!paths.contains(&"bad.ts".to_string()), "gitignored dir should be excluded");
        assert!(!paths.contains(&"pkg.ts".to_string()), "node_modules should be excluded");
    }
}
