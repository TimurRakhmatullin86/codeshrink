mod parser;
mod query;
mod ranker;
mod extractor;
mod output;

pub use parser::{Language, ParsedFile, Symbol, SymbolKind};
pub use query::QueryTerms;
pub use ranker::RankedSymbol;
pub use extractor::ExtractedContext;
pub use output::OutputFormat;

use std::path::Path;

#[derive(Debug, Clone)]
pub struct ShrinkOptions {
    pub context_lines: usize,
    pub max_symbols: usize,
    pub format: OutputFormat,
    pub include_imports: bool,
}

impl Default for ShrinkOptions {
    fn default() -> Self {
        Self {
            context_lines: 5,
            max_symbols: 50,
            format: OutputFormat::Markdown,
            include_imports: true,
        }
    }
}

#[derive(Debug)]
pub struct ShrinkResult {
    pub compressed: String,
    pub stats: ShrinkStats,
}

#[derive(Debug)]
pub struct ShrinkStats {
    pub files_scanned: usize,
    pub files_matched: usize,
    pub symbols_found: usize,
    pub symbols_returned: usize,
    pub input_lines: usize,
    pub output_lines: usize,
    pub compression_ratio: f64,
}

pub fn shrink(query: &str, repo_path: &Path, opts: &ShrinkOptions) -> Result<ShrinkResult, ShrinkError> {
    let terms = query::extract_terms(query);
    if terms.is_empty() {
        return Err(ShrinkError::EmptyQuery);
    }

    let files = parser::scan_repo(repo_path)?;
    if files.is_empty() {
        return Err(ShrinkError::NoFiles);
    }

    let total_lines: usize = files.iter().map(|f| f.line_count).sum();

    let ranked = ranker::rank_symbols(&files, &terms, opts.max_symbols);

    let extracted = extractor::extract_contexts(&files, &ranked, opts.context_lines, opts.include_imports);

    let compressed = output::format_output(&extracted, &terms, opts.format);

    let output_lines = compressed.lines().count();
    let ratio = if total_lines > 0 {
        1.0 - (output_lines as f64 / total_lines as f64)
    } else {
        0.0
    };

    Ok(ShrinkResult {
        compressed,
        stats: ShrinkStats {
            files_scanned: files.len(),
            files_matched: extracted.len(),
            symbols_found: files.iter().map(|f| f.symbols.len()).sum(),
            symbols_returned: ranked.len(),
            input_lines: total_lines,
            output_lines,
            compression_ratio: ratio,
        },
    })
}

#[derive(Debug, thiserror::Error)]
pub enum ShrinkError {
    #[error("empty query — provide at least one search term")]
    EmptyQuery,
    #[error("no parseable files found")]
    NoFiles,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
