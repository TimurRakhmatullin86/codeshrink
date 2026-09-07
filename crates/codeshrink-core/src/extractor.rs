use crate::parser::ParsedFile;
use crate::ranker::RankedSymbol;


#[derive(Debug, Clone)]
pub struct ExtractedContext {
    pub file_path: String,
    pub imports: Vec<String>,
    pub snippets: Vec<Snippet>,
}

#[derive(Debug, Clone)]
pub struct Snippet {
    pub symbol_name: String,
    pub signature: String,
    pub start_line: usize,
    pub end_line: usize,
    pub code: String,
    pub score: f64,
}

pub fn extract_contexts(
    files: &[ParsedFile],
    ranked: &[RankedSymbol],
    context_lines: usize,
    include_imports: bool,
) -> Vec<ExtractedContext> {
    // Group ranked symbols by file
    let mut file_groups: std::collections::BTreeMap<usize, Vec<&RankedSymbol>> =
        std::collections::BTreeMap::new();
    for rs in ranked {
        file_groups.entry(rs.file_index).or_default().push(rs);
    }

    let mut results = Vec::new();

    for (fi, symbols) in &file_groups {
        let file = match files.get(*fi) {
            Some(f) => f,
            None => continue,
        };

        let lines: Vec<&str> = file.source.lines().collect();

        let imports = if include_imports {
            file.imports
                .iter()
                .map(|imp| imp.signature.clone())
                .collect()
        } else {
            vec![]
        };

        let mut snippets = Vec::new();

        // Collect all line ranges, merging overlaps
        let mut ranges: Vec<(usize, usize, usize)> = Vec::new(); // (start, end, sym_idx)
        for rs in symbols {
            let sym = match file.symbols.get(rs.symbol_index) {
                Some(s) => s,
                None => continue,
            };

            let start = sym.start_line.saturating_sub(context_lines);
            let end = (sym.end_line + context_lines).min(lines.len().saturating_sub(1));
            ranges.push((start, end, rs.symbol_index));
        }

        // Sort by start line
        ranges.sort_by_key(|r| r.0);

        // Merge overlapping ranges
        let mut merged: Vec<(usize, usize, Vec<usize>)> = Vec::new();
        for (start, end, si) in ranges {
            if let Some(last) = merged.last_mut() {
                if start <= last.1 + 1 {
                    last.1 = last.1.max(end);
                    last.2.push(si);
                    continue;
                }
            }
            merged.push((start, end, vec![si]));
        }

        for (start, end, sym_indices) in merged {
            let code: String = lines[start..=end.min(lines.len() - 1)]
                .iter()
                .enumerate()
                .map(|(i, line)| format!("{:>4} | {}", start + i + 1, line))
                .collect::<Vec<_>>()
                .join("\n");

            let primary_sym = sym_indices
                .first()
                .and_then(|&si| file.symbols.get(si));

            let best_score = symbols
                .iter()
                .filter(|rs| sym_indices.contains(&rs.symbol_index))
                .map(|rs| rs.score)
                .fold(0.0f64, f64::max);

            snippets.push(Snippet {
                symbol_name: primary_sym.map(|s| s.name.clone()).unwrap_or_default(),
                signature: primary_sym.map(|s| s.signature.clone()).unwrap_or_default(),
                start_line: start,
                end_line: end,
                code,
                score: best_score,
            });
        }

        let rel_path = file.path.to_string_lossy().to_string();

        results.push(ExtractedContext {
            file_path: rel_path,
            imports,
            snippets,
        });
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Language, ParsedFile, Symbol, SymbolKind};
    use crate::ranker::RankedSymbol;
    use std::path::PathBuf;

    #[test]
    fn extracts_context_with_surrounding_lines() {
        let source = (0..20)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");

        let files = vec![ParsedFile {
            path: PathBuf::from("test.ts"),
            language: Language::TypeScript,
            source,
            line_count: 20,
            symbols: vec![Symbol {
                name: "target".to_string(),
                kind: SymbolKind::Function,
                start_line: 10,
                end_line: 12,
                signature: "function target()".to_string(),
            }],
            imports: vec![],
        }];

        let ranked = vec![RankedSymbol {
            file_index: 0,
            symbol_index: 0,
            score: 1.0,
        }];

        let ctx = extract_contexts(&files, &ranked, 3, false);
        assert_eq!(ctx.len(), 1);
        assert_eq!(ctx[0].snippets.len(), 1);
        assert_eq!(ctx[0].snippets[0].start_line, 7);
        assert_eq!(ctx[0].snippets[0].end_line, 15);
    }

    #[test]
    fn merges_overlapping_ranges() {
        let source = (0..30)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");

        let files = vec![ParsedFile {
            path: PathBuf::from("test.ts"),
            language: Language::TypeScript,
            source,
            line_count: 30,
            symbols: vec![
                Symbol {
                    name: "func_a".to_string(),
                    kind: SymbolKind::Function,
                    start_line: 5,
                    end_line: 8,
                    signature: "function func_a()".to_string(),
                },
                Symbol {
                    name: "func_b".to_string(),
                    kind: SymbolKind::Function,
                    start_line: 10,
                    end_line: 13,
                    signature: "function func_b()".to_string(),
                },
            ],
            imports: vec![],
        }];

        let ranked = vec![
            RankedSymbol { file_index: 0, symbol_index: 0, score: 1.0 },
            RankedSymbol { file_index: 0, symbol_index: 1, score: 0.8 },
        ];

        let ctx = extract_contexts(&files, &ranked, 3, false);
        assert_eq!(ctx[0].snippets.len(), 1, "overlapping ranges should merge");
    }
}
