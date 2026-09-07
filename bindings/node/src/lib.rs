use napi_derive::napi;
use std::path::Path;

#[napi(object)]
pub struct JsShrinkOptions {
    pub context_lines: Option<u32>,
    pub max_symbols: Option<u32>,
    pub format: Option<String>,
    pub include_imports: Option<bool>,
}

#[napi(object)]
pub struct JsShrinkStats {
    pub files_scanned: u32,
    pub files_matched: u32,
    pub symbols_found: u32,
    pub symbols_returned: u32,
    pub input_lines: u32,
    pub output_lines: u32,
    pub compression_ratio: f64,
}

#[napi(object)]
pub struct JsShrinkResult {
    pub compressed: String,
    pub stats: JsShrinkStats,
}

#[napi]
pub fn shrink(query: String, path: String, options: Option<JsShrinkOptions>) -> napi::Result<JsShrinkResult> {
    let opts = match options {
        Some(o) => codeshrink_core::ShrinkOptions {
            context_lines: o.context_lines.unwrap_or(5) as usize,
            max_symbols: o.max_symbols.unwrap_or(50) as usize,
            format: match o.format.as_deref() {
                Some("xml") => codeshrink_core::OutputFormat::Xml,
                Some("plain") => codeshrink_core::OutputFormat::Plain,
                _ => codeshrink_core::OutputFormat::Markdown,
            },
            include_imports: o.include_imports.unwrap_or(true),
        },
        None => codeshrink_core::ShrinkOptions::default(),
    };

    let result = codeshrink_core::shrink(&query, Path::new(&path), &opts)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    Ok(JsShrinkResult {
        compressed: result.compressed,
        stats: JsShrinkStats {
            files_scanned: result.stats.files_scanned as u32,
            files_matched: result.stats.files_matched as u32,
            symbols_found: result.stats.symbols_found as u32,
            symbols_returned: result.stats.symbols_returned as u32,
            input_lines: result.stats.input_lines as u32,
            output_lines: result.stats.output_lines as u32,
            compression_ratio: result.stats.compression_ratio,
        },
    })
}
