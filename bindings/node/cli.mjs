#!/usr/bin/env node

import { shrink } from './index.js';
import { resolve } from 'path';

const args = process.argv.slice(2);

if (args.length === 0 || args.includes('--help') || args.includes('-h')) {
    console.log(`codeshrink — query-aware code compression for LLMs

Usage: codeshrink <query> [options]

Options:
  --path <dir>       Repository path (default: .)
  --context <n>      Context lines around symbols (default: 5)
  --max-symbols <n>  Maximum symbols to return (default: 50)
  --format <fmt>     Output format: markdown, xml, plain (default: markdown)
  --no-imports       Exclude import statements
  --stats-only       Print stats only, no compressed output
  -h, --help         Show this help`);
    process.exit(0);
}

const query = args[0];
let repoPath = '.';
let contextLines = 5;
let maxSymbols = 50;
let format = 'markdown';
let includeImports = true;
let statsOnly = false;

for (let i = 1; i < args.length; i++) {
    switch (args[i]) {
        case '--path': repoPath = args[++i]; break;
        case '--context': contextLines = parseInt(args[++i], 10); break;
        case '--max-symbols': maxSymbols = parseInt(args[++i], 10); break;
        case '--format': format = args[++i]; break;
        case '--no-imports': includeImports = false; break;
        case '--stats-only': statsOnly = true; break;
    }
}

try {
    const start = performance.now();
    const result = shrink(query, resolve(repoPath), { contextLines, maxSymbols, format, includeImports });
    const elapsed = performance.now() - start;

    if (!statsOnly) {
        process.stdout.write(result.compressed);
    }

    const { stats } = result;
    const ratio = (stats.compressionRatio * 100).toFixed(1);
    process.stderr.write(`\n--- codeshrink stats ---
Files: ${stats.filesScanned} scanned, ${stats.filesMatched} matched
Symbols: ${stats.symbolsFound} found, ${stats.symbolsReturned} returned
Lines: ${stats.inputLines} → ${stats.outputLines} (${ratio}% compression)
Time: ${elapsed.toFixed(1)}ms
`);
} catch (e) {
    console.error(`error: ${e.message}`);
    process.exit(1);
}
