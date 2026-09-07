use crate::parser::ParsedFile;
use crate::query::QueryTerms;

#[derive(Debug, Clone)]
pub struct RankedSymbol {
    pub file_index: usize,
    pub symbol_index: usize,
    pub score: f64,
}

pub fn rank_symbols(
    files: &[ParsedFile],
    terms: &QueryTerms,
    max_symbols: usize,
) -> Vec<RankedSymbol> {
    let mut scored: Vec<RankedSymbol> = Vec::new();

    for (fi, file) in files.iter().enumerate() {
        let path_str = file.path.to_string_lossy().to_lowercase();
        let path_boost = if terms.exact.iter().any(|t| path_str.contains(t)) {
            0.2
        } else {
            0.0
        };

        for (si, sym) in file.symbols.iter().enumerate() {
            let name_score = terms.matches(&sym.name);
            let sig_score = terms.matches(&sym.signature) * 0.5;

            let total = name_score.max(sig_score) + path_boost;

            if total > 0.0 {
                scored.push(RankedSymbol {
                    file_index: fi,
                    symbol_index: si,
                    score: total,
                });
            }
        }
    }

    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    // After direct matches, add callers/importers of top symbols (1-hop expansion)
    let top_names: Vec<String> = scored
        .iter()
        .take(max_symbols / 2)
        .filter_map(|r| {
            files.get(r.file_index)
                .and_then(|f| f.symbols.get(r.symbol_index))
                .map(|s| s.name.clone())
        })
        .collect();

    if !top_names.is_empty() {
        for (fi, file) in files.iter().enumerate() {
            for (si, sym) in file.symbols.iter().enumerate() {
                if scored.iter().any(|r| r.file_index == fi && r.symbol_index == si) {
                    continue;
                }
                let sig_lower = sym.signature.to_lowercase();
                for name in &top_names {
                    if sig_lower.contains(&name.to_lowercase()) {
                        scored.push(RankedSymbol {
                            file_index: fi,
                            symbol_index: si,
                            score: 0.2,
                        });
                        break;
                    }
                }
            }
        }
    }

    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(max_symbols);
    scored
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Language, Symbol, SymbolKind};
    use std::path::PathBuf;

    fn make_file(path: &str, symbols: Vec<(&str, &str)>) -> ParsedFile {
        ParsedFile {
            path: PathBuf::from(path),
            language: Language::TypeScript,
            source: String::new(),
            line_count: 100,
            symbols: symbols
                .into_iter()
                .enumerate()
                .map(|(i, (name, sig))| Symbol {
                    name: name.to_string(),
                    kind: SymbolKind::Function,
                    start_line: i * 10,
                    end_line: i * 10 + 8,
                    signature: sig.to_string(),
                })
                .collect(),
            imports: vec![],
        }
    }

    #[test]
    fn ranks_direct_match_highest() {
        let files = vec![
            make_file("src/auth.ts", vec![
                ("authenticate", "function authenticate(req: Request)"),
                ("unrelated", "function unrelated()"),
            ]),
            make_file("src/db.ts", vec![
                ("getUser", "function getUser(id: string)"),
            ]),
        ];
        let terms = crate::query::extract_terms("authenticate");
        let ranked = rank_symbols(&files, &terms, 10);

        assert!(!ranked.is_empty());
        assert_eq!(ranked[0].file_index, 0);
        assert_eq!(ranked[0].symbol_index, 0);
        assert!(ranked[0].score > 0.5);
    }

    #[test]
    fn path_boost_works() {
        let files = vec![
            make_file("src/auth/middleware.ts", vec![
                ("validateToken", "function validateToken(t: string)"),
            ]),
            make_file("src/utils/helper.ts", vec![
                ("validateInput", "function validateInput(x: any)"),
            ]),
        ];
        let terms = crate::query::extract_terms("auth");
        let ranked = rank_symbols(&files, &terms, 10);

        let auth_entry = ranked.iter().find(|r| r.file_index == 0);
        let util_entry = ranked.iter().find(|r| r.file_index == 1);

        assert!(auth_entry.is_some(), "auth file should have matches due to path boost");
        let auth_score = auth_entry.unwrap().score;
        let util_score = util_entry.map(|r| r.score).unwrap_or(0.0);
        assert!(auth_score > util_score, "auth path should boost: {} vs {}", auth_score, util_score);
    }
}
