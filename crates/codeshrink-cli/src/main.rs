use clap::Parser;
use codeshrink_core::{shrink, OutputFormat, ShrinkOptions};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(
    name = "codeshrink",
    about = "Query-aware AST code compression for LLMs",
    long_about = "Feed a question + repo path → get only the relevant code snippets.\n\
                  Tree-sitter powered, not embeddings. 80k LOC → 1.2k in 20ms.",
    version
)]
struct Cli {
    /// The query to search for (e.g. "where is authentication?")
    query: String,

    /// Path to the repository (default: current directory)
    #[arg(short, long, default_value = ".")]
    path: PathBuf,

    /// Lines of context around each matched symbol
    #[arg(short, long, default_value = "5")]
    context: usize,

    /// Maximum number of symbols to return
    #[arg(short, long, default_value = "50")]
    max_symbols: usize,

    /// Output format: markdown, xml, plain
    #[arg(short, long, default_value = "markdown")]
    format: String,

    /// Show statistics only (no code output)
    #[arg(long)]
    stats_only: bool,

    /// Exclude import statements from output
    #[arg(long)]
    no_imports: bool,
}

fn main() {
    let cli = Cli::parse();

    let format = match cli.format.as_str() {
        "xml" => OutputFormat::Xml,
        "plain" => OutputFormat::Plain,
        _ => OutputFormat::Markdown,
    };

    let opts = ShrinkOptions {
        context_lines: cli.context,
        max_symbols: cli.max_symbols,
        format,
        include_imports: !cli.no_imports,
    };

    let start = Instant::now();

    match shrink(&cli.query, &cli.path, &opts) {
        Ok(result) => {
            let elapsed = start.elapsed();

            if !cli.stats_only {
                print!("{}", result.compressed);
            }

            eprintln!();
            eprintln!("--- codeshrink stats ---");
            eprintln!(
                "Files: {} scanned, {} matched",
                result.stats.files_scanned, result.stats.files_matched
            );
            eprintln!(
                "Symbols: {} found, {} returned",
                result.stats.symbols_found, result.stats.symbols_returned
            );
            eprintln!(
                "Lines: {} → {} ({:.1}% compression)",
                result.stats.input_lines,
                result.stats.output_lines,
                result.stats.compression_ratio * 100.0
            );
            eprintln!("Time: {:.1}ms", elapsed.as_secs_f64() * 1000.0);
        }
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}
