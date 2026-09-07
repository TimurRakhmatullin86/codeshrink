use crate::extractor::ExtractedContext;
use crate::query::QueryTerms;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Markdown,
    Xml,
    Plain,
}

pub fn format_output(
    contexts: &[ExtractedContext],
    terms: &QueryTerms,
    format: OutputFormat,
) -> String {
    match format {
        OutputFormat::Markdown => format_markdown(contexts, terms),
        OutputFormat::Xml => format_xml(contexts, terms),
        OutputFormat::Plain => format_plain(contexts, terms),
    }
}

fn format_markdown(contexts: &[ExtractedContext], terms: &QueryTerms) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# CodeShrink: query={:?}\n\n",
        terms.exact.join(" ")
    ));

    for ctx in contexts {
        out.push_str(&format!("## {}\n\n", ctx.file_path));

        if !ctx.imports.is_empty() {
            out.push_str("```\n");
            for imp in &ctx.imports {
                out.push_str(imp);
                out.push('\n');
            }
            out.push_str("```\n\n");
        }

        for snippet in &ctx.snippets {
            if !snippet.symbol_name.is_empty() {
                out.push_str(&format!(
                    "### `{}` (score: {:.2})\n\n",
                    snippet.symbol_name, snippet.score
                ));
            }
            out.push_str("```\n");
            out.push_str(&snippet.code);
            out.push_str("\n```\n\n");
        }
    }

    out
}

fn format_xml(contexts: &[ExtractedContext], terms: &QueryTerms) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "<codeshrink query=\"{}\">\n",
        terms.exact.join(" ")
    ));

    for ctx in contexts {
        out.push_str(&format!("  <file path=\"{}\">\n", ctx.file_path));

        if !ctx.imports.is_empty() {
            out.push_str("    <imports>\n");
            for imp in &ctx.imports {
                out.push_str(&format!("      {}\n", imp));
            }
            out.push_str("    </imports>\n");
        }

        for snippet in &ctx.snippets {
            out.push_str(&format!(
                "    <symbol name=\"{}\" score=\"{:.2}\" lines=\"{}-{}\">\n",
                snippet.symbol_name, snippet.score, snippet.start_line + 1, snippet.end_line + 1
            ));
            for line in snippet.code.lines() {
                out.push_str(&format!("      {}\n", line));
            }
            out.push_str("    </symbol>\n");
        }

        out.push_str("  </file>\n");
    }

    out.push_str("</codeshrink>\n");
    out
}

fn format_plain(contexts: &[ExtractedContext], _terms: &QueryTerms) -> String {
    let mut out = String::new();

    for ctx in contexts {
        out.push_str(&format!("=== {} ===\n", ctx.file_path));

        if !ctx.imports.is_empty() {
            for imp in &ctx.imports {
                out.push_str(imp);
                out.push('\n');
            }
            out.push('\n');
        }

        for snippet in &ctx.snippets {
            out.push_str(&snippet.code);
            out.push_str("\n\n");
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::{ExtractedContext, Snippet};

    fn sample_context() -> Vec<ExtractedContext> {
        vec![ExtractedContext {
            file_path: "src/auth.ts".to_string(),
            imports: vec!["import { Request } from 'express';".to_string()],
            snippets: vec![Snippet {
                symbol_name: "authenticate".to_string(),
                signature: "function authenticate(req: Request)".to_string(),
                start_line: 2,
                end_line: 8,
                code: "   3 | function authenticate(req: Request) {\n   4 |     return true;\n   5 | }".to_string(),
                score: 1.0,
            }],
        }]
    }

    #[test]
    fn markdown_output_contains_file_and_symbol() {
        let terms = crate::query::extract_terms("auth");
        let out = format_markdown(&sample_context(), &terms);
        assert!(out.contains("src/auth.ts"));
        assert!(out.contains("authenticate"));
        assert!(out.contains("import { Request }"));
    }

    #[test]
    fn xml_output_is_structured() {
        let terms = crate::query::extract_terms("auth");
        let out = format_xml(&sample_context(), &terms);
        assert!(out.starts_with("<codeshrink"));
        assert!(out.contains("<file path=\"src/auth.ts\">"));
        assert!(out.contains("<symbol name=\"authenticate\""));
        assert!(out.ends_with("</codeshrink>\n"));
    }

    #[test]
    fn plain_output_minimal() {
        let terms = crate::query::extract_terms("auth");
        let out = format_plain(&sample_context(), &terms);
        assert!(out.contains("=== src/auth.ts ==="));
        assert!(out.contains("function authenticate"));
    }
}
